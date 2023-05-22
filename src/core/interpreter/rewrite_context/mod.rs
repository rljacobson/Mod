/*!

Holds state for a rewrite system.

Maude uses inheritance: A `RewritingContext` is-a `Substitution`. We use composition: A `RewritingContext` has-a
`Substitution`. Maude source says:

> A rewriting context keeps track of miscellaneous information needed while rewriting. An important performance trick is
> that we derive it from Substitution so that we can use the rewriting context to construct matching substitutions in.
> This avoids creating a new substitution at the start of each match attempt.

I interpret this to mean that the Substitution data structure is reused between matches instead of created and
destroyed.

ToDo: This implements way more of Maude than a pattern matching library should have. Refactor to remove
      application-specific infrastructure.

*/

pub mod trace;
mod trial;
mod context_attributes;
mod debugger;

use std::rc::Weak;

use crate::{
  abstractions::{
    RcCell,
    WeakCell,
  },
  core::{
    condition_fragment::ConditionFragment,
    Equation,
    interpreter::{
      Interpreter,
      InterpreterAttribute,
      tui::TUI,
      WeakInterpreter,
    },
    NarrowingVariableInfo,
    RedexPosition,
    sort::SortConstraint,
    StrategyDefinition,
    substitution::Substitution,
  },
  ROOT_OK,
  theory::RcDagNode,
  UNDEFINED,
};

pub use crate::core::rewrite_context::context_attributes::{ContextAttribute, ContextAttributes};


pub type RcRewritingContext = RcCell<RewritingContext>;
pub type WeakRewritingContext = WeakCell<RewritingContext>;


const HEADER: &str = "*********** ";

pub enum Purpose {
  ConditionEval,
  SortEval,
  TopLevelEval,
  MetaEval,
  Other
}

pub enum RewriteType {
  Normal,
  Builtin,
  Memoized,
}


pub(crate) struct RewritingContext {
  // Base class members

  // progress: bool, // Only used for object system

  /// Intended for garbage collection
  root: RcDagNode,

  /// Statistics, records how many rewrites were done.
  mb_count: u64, // Membership
  eq_count: u64, // Equation
  rl_count: u64, // Rule

  narrowing_count        : u64,
  variant_narrowing_count: u64,

  //	For rule rewriting
  redex_stack  : Vec<RedexPosition>,
  stale_marker : i32, // NONE = -1, ROOT_OK = -2, an index when >= 0
  lazy_marker  : i32, // NONE = -1, an index when >= 0
  current_index: i32,

  // rewrite_limit: u64, // NONE = -1, Only used for object system
  // gas_per_node : u64, // Only used for object system
  // current_gas  : u64, // Only used for object system

  // "User Level" members

  parent          : Option<WeakRewritingContext>,
  interpreter     : WeakInterpreter,
  substitution    : Substitution,
  purpose         : Purpose,
  trial_count     : usize,
  attributes      : ContextAttributes,
  debug_level     : i32,

  // Do not belong here
  tui: TUI,
}

impl RewritingContext {
  pub fn new(root: RcDagNode, interpreter: WeakInterpreter) -> Self {
    RewritingContext {
      root,
      mb_count               : 0,
      eq_count               : 0,
      rl_count               : 0,
      narrowing_count        : 0,
      variant_narrowing_count: 0,
      redex_stack            : vec![],
      stale_marker           : 0,
      lazy_marker            : 0,
      current_index          : 0,
      parent                 : None,
      interpreter,
      substitution           : Substitution::default(),
      purpose                : Purpose::TopLevelEval,
      trial_count            : 0,
      attributes             : ContextAttributes::default(),
      debug_level            : UNDEFINED,
      tui                    : TUI::default(),
    }
  }

  pub fn with_parent(
    root            : RcDagNode,
    parent          : Option<WeakRewritingContext>,
    purpose         : Purpose,
    local_trace_flag: bool, // ToDo: Change to `ContextAttributes`?
    interpreter     : WeakInterpreter,
  ) -> Self {
    RewritingContext {
      root,
      eq_count       : 0,
      mb_count       : 0,
      narrowing_count: 0,
      rl_count       : 0,
      variant_narrowing_count: 0,
      redex_stack  : vec![],
      stale_marker : 0,
      lazy_marker  : 0,
      current_index: 0,
      parent,
      interpreter,
      substitution: Default::default(),
      purpose,
      trial_count : 0,
      attributes  : if local_trace_flag {
                      ContextAttribute::LocalTrace.into()
                    } else {
                      ContextAttributes::default()
                    },
      debug_level : UNDEFINED,
      tui         : TUI::default(),
    }
  }

  #[inline(always)]
  pub fn attribute(&self, attribute: ContextAttribute) -> bool {
    self.attributes.has_attribute(attribute)
  }


  // region Statistics
  #[inline(always)]
  fn clear_counts(&mut self) {
    self.mb_count = 0;
    self.eq_count = 0;
    self.rl_count = 0;
    self.narrowing_count = 0;
    self.variant_narrowing_count = 0;
  }

  #[inline(always)]
  fn add_counts_from(&mut self, other: &RewritingContext) {

    self.mb_count += other.mb_count;
    self.eq_count += other.eq_count;
    self.rl_count += other.rl_count;
    self.narrowing_count += other.narrowing_count;
    self.variant_narrowing_count += other.variant_narrowing_count;
  }

  #[inline(always)]
  fn transfer_counts_from(&mut self, other: &mut RewritingContext) {
    self.add_counts_from(other);
    other.clear_counts();
  }
  // endregion


  // region Rebuilding Stale DagNodes

  fn rebuild_upto_root(&mut self) {
    let mut i: i32;
    let mut current_idx: i32;
    // println!("\nroot was {:?}", self.root_node);
    // println!("rebuilding from {}", self.current_index);
    assert!(self.current_index >= 0, "bad currentIndex");

    // Locate deepest stack node with a stale parent.
    current_idx = self.current_index; // All staleness guaranteed to be above current_index
    while self.redex_stack[current_idx as usize].parent_index != self.stale_marker {
      current_idx = self.redex_stack[current_idx as usize].parent_index;
    }

    // We assume that we only have to rebuild the spine from staleMarker to root.
    i = self.stale_marker;

    while i != UNDEFINED {
      self.remake_stale_dag_node(i, current_idx);
      current_idx = i;
      i = self.redex_stack[i as usize].parent_index;
    }

    self.root = self.redex_stack[0].dag_node.clone();
    self.stale_marker = ROOT_OK;

    // println!("root is {:?}", self.root_node);
  }

  fn remake_stale_dag_node(&mut self, stale_index: i32, child_index: i32) {
    // Find first stacked argument of stale dag node.
    let mut first_idx = child_index as usize;
    while self.redex_stack[first_idx - 1].parent_index == stale_index {
      first_idx -= 1;
    }

    // Find last stacked argument of stale dag node.
    let mut last_idx = child_index as usize;
    let stack_length = self.redex_stack.len();
    while last_idx + 1 < stack_length && self.redex_stack[last_idx + 1].parent_index == stale_index {
      last_idx += 1;
    }

    // Replace stale dag node with a copy in which stacked arguments
    // replace corresponding arguments in the original.
    let remade = self.redex_stack[stale_index as usize]
                     .dag_node.borrow()
                     .copy_with_replacements(&self.redex_stack, first_idx, last_idx);
    self.redex_stack[stale_index as usize].dag_node = remade;
  }

  // endregion

}


pub fn make_subcontext(parent: RcRewritingContext, root: RcDagNode, purpose: Purpose) -> RewritingContext {
  let parent_ref = parent.borrow();

  RewritingContext::with_parent(
    root,
    Some(parent.downgrade()),
    purpose,
    parent_ref.attribute(ContextAttribute::LocalTrace),
    parent_ref.interpreter.clone()
  )
}
