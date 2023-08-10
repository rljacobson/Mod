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
mod context_attributes;
pub(crate) mod debugger;

use std::fmt::{Display, Formatter};
use std::rc::Weak;

use crate::{
  abstractions::{
    RcCell,
    WeakCell,
  },
  core::{
    condition_fragment::ConditionFragment,
    interpreter::{
      Interpreter,
      InterpreterAttribute,
      tui::TUI,
      WeakInterpreter,
    },
    NarrowingVariableInfo,
    RedexPosition,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Purpose {
  ConditionEval,
  SortEval,
  TopLevelEval,
  MetaEval,
  Other
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RewriteType {
  Normal,
  Builtin,
  Memoized,
}

impl Display for RewriteType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      RewriteType::Normal => {
        write!(f, "normal")
      }
      RewriteType::Builtin => {
        write!(f, "built-in")
      }
      RewriteType::Memoized => {
        write!(f, "memoized")
      }
    }
  }
}


pub(crate) struct RewritingContext {
  // Base class members

  // progress: bool, // Only used for object system

  pub(crate) root: Option<RcDagNode>,

  /// Statistics, records how many rewrites were done.
  pub(crate) mb_count: u64, // Membership
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
  pub(crate) substitution    : Substitution,
  purpose         : Purpose,
  trial_count     : usize,
  attributes      : ContextAttributes,
  debug_level     : i32,

  // Do not belong here
  tui: TUI,
}

impl RewritingContext {
  pub fn new(root: Option<RcDagNode>, interpreter: WeakInterpreter) -> Self {
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
    root            : Option<RcDagNode>,
    parent          : Option<WeakRewritingContext>,
    purpose         : Purpose,
    local_trace_flag: bool,
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

  /// A limited RewritingContext:
  ///  1. Does not have a rootNode.
  ///  2. Need not have a substitution large enough to apply sort constraints.
  ///  3. ~Does not protect its substitution from garbage collection.~
  ///  4. ~Does not protect its redex stack from garbage collection.~
  /// It exists so that certain functions that expect a RewritingContext,
  /// ultimately to compute true sorts by applying sort constraints can be
  /// called by unification code when a general purpose RewritingContext
  /// not available. Sort constraints are not supported by unification and
  /// are thus ignored if the supplied RewritingContext is limited.
  #[inline(always)]
  pub fn is_limited(&self) -> bool {
    self.root.is_none()
  }

  #[inline(always)]
  pub fn trace_abort(&self) -> bool {
    self.attribute(ContextAttribute::Abort)
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
  pub fn add_counts_from(&mut self, other: &RewritingContext) {

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

    self.root = Some(self.redex_stack[0].dag_node.clone());
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


  #[inline(always)]
  pub fn finished(&mut self) {
    self.substitution.finished()
  }

  #[inline(always)]
  pub fn reduce(&mut self) {
    if let Some(root) = &self.root {
      // root.borrow_mut().reduce(self);
      // let root = root.borrow_mut();

      self.reduce_dag_node(root.clone());
    }
  }

  #[inline(always)]
  pub fn reduce_dag_node(&mut self, dag_node: RcDagNode) {
    while !dag_node.borrow().is_reduced() {
      let mut symbol = dag_node.borrow().symbol();

      if !(symbol.rewrite(dag_node.clone(), self)) {
        dag_node.borrow_mut().set_reduced();
        self.fast_compute_true_sort(dag_node.clone());
      }
    }
  }

  /// Computes the true sort of root.
  #[inline(always)]
  fn fast_compute_true_sort(&mut self, dag_node: RcDagNode) {
    // let root = self.root.unwrap();
    let t = dag_node.borrow()
                .symbol()
                .symbol_members()
                .unique_sort_index;

    if t < 0 {
      dag_node.borrow_mut().compute_base_sort();  // usual case
    }
    else if t > 0 {
      dag_node.borrow_mut().set_sort_index(t);  // unique sort case
    }
    else {
      self.slow_compute_true_sort(dag_node);  // most general case
    }
  }

  /// Computes the true sort of root.
  fn slow_compute_true_sort(&mut self, dag_node: RcDagNode) {
    // let root = self.root.unwrap();
    let mut symbol = dag_node.borrow_mut().symbol();
    symbol.sort_constraint_table()
        .constrain_to_smaller_sort(
          dag_node.clone(),
          self
        );
  }

}


pub fn make_subcontext(parent: RcRewritingContext, root: Option<RcDagNode>, purpose: Purpose) -> RewritingContext {
  let parent_ref = parent.borrow();

  RewritingContext::with_parent(
    root,
    Some(parent.downgrade()),
    purpose,
    parent_ref.attribute(ContextAttribute::LocalTrace),
    parent_ref.interpreter.clone()
  )
}
