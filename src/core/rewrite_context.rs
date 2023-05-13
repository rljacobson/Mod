/*!

Holds state for a rewrite system.

Maude uses inheritance: A `RewritingContext` is-a `Substitution`. We use composition: A `RewritingContext` has-a
`Substitution`. Maude source says:

> A rewriting context keeps track of miscellaneous information needed while rewriting. An important performance trick is
> that we derive it from Substitution so that we can use the rewriting context to construct matching substitutions in.
> This avoids creating a new substitution at the start of each match attempt.

I interpret this to mean that the Substitution data structure is reused between matches instead of created and
destroyed.
*/

use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
  core::{
    Equation,
    NarrowingVariableInfo,
    PreEquation,
    RedexPosition,
    Rule,
    SortConstraint,
    StrategyDefinition,
    Substitution
  },
  theory::RcDagNode,
  ROOT_OK,
  UNDEFINED,
};
use crate::core::ConditionFragment;


/// Tracing status is global for all `RewritingContext`s.
static TRACE_STATUS: AtomicBool = AtomicBool::new(false);


/// Tracing status is global for all `RewritingContext`s.
pub fn trace_status() -> bool {
  TRACE_STATUS.load(Ordering::Relaxed)
}
/// Tracing status is global for all `RewritingContext`s.
pub fn set_trace_status(status: bool) {
  TRACE_STATUS.store(status, Ordering::Relaxed);
}

pub enum Purpose {
  ConditionEval,
  SortEval,
  Other
}

pub enum RewriteType {
  Normal,
  Builtin,
  Memoized
}


pub struct RewritingContextMembers {
  pub solution: Substitution,

  // progress: bool, // Only used for object system

  /// Intended for garbage collection
  root_node: RcDagNode,

  /// Statistics, records how many rewrites were done.
  mb_count: u64, // Match something?
  eq_count: u64, // Equation count?
  rl_count: u64, // Rule count?

  narrowing_count        : u64, // ?
  variant_narrowing_count: u64, // ?


  //	For rule rewriting
  redex_stack  : Vec<RedexPosition>,
  stale_marker : i32, // NONE = -1, ROOT_OK = -2, an index when >= 0
  lazy_marker  : i32, // NONE = -1, an index when >= 0
  current_index: i32,

  // rewrite_limit: u64, // NONE = -1, Only used for object system
  // gas_per_node : u64, // Only used for object system
  // current_gas  : u64, // Only used for object system
}


pub trait RewritingContext {
  fn members(&self) -> &RewritingContextMembers;
  fn members_mut(&self) -> &mut RewritingContextMembers;

  // region Statistics
  #[inline(always)]
  fn clear_counts(&mut self) {
    let mut members = self.members_mut();
    members.mb_count = 0;
    members.eq_count = 0;
    members.rl_count = 0;
    members.narrowing_count = 0;
    members.variant_narrowing_count = 0;
  }

  #[inline(always)]
  fn add_counts_from(&mut self, other: &dyn RewritingContext) {
    let mut members = self.members_mut();
    let other_members = other.members();

    members.mb_count += other_members.mb_count;
    members.eq_count += other_members.eq_count;
    members.rl_count += other_members.rl_count;
    members.narrowing_count += other_members.narrowing_count;
    members.variant_narrowing_count += other_members.variant_narrowing_count;
  }

  #[inline(always)]
  fn transfer_counts_from(&mut self, other: &mut dyn RewritingContext) {
    self.add_counts_from(other);
    other.clear_counts();
  }
  // endregion

  // region Tracing methods


  fn trace_begin_eq_trial(&mut self, _subject: RcDagNode, _equation: &Equation) -> i32 {
    0
  }

  fn trace_begin_rule_trial(&mut self, _subject: RcDagNode, _rule: &Rule) -> i32 {
    0
  }

  fn trace_begin_sc_trial(&mut self, _subject: RcDagNode, _sc: &SortConstraint) -> i32 {
    0
  }

  fn trace_begin_sd_trial(&mut self, _subject: RcDagNode, _sd: &StrategyDefinition) -> i32 {
    0
  }

  fn trace_end_trial(&mut self, _trial_ref: &mut Option<i32>, _success: bool) {
    // Empty default implementation
  }

  fn trace_exhausted(&mut self, _trial_ref: &mut Option<i32>) {
    // Empty default implementation
  }

  fn trace_pre_eq_rewrite(&mut self, _redex: RcDagNode, _equation: &Equation, _type: i32) {
    // Empty default implementation
  }

  fn trace_post_eq_rewrite(&mut self, _replacement: RcDagNode) {
    // Empty default implementation
  }

  fn trace_pre_rule_rewrite(&mut self, _redex: RcDagNode, _rule: &Rule) {
    // Empty default implementation
  }

  fn trace_post_rule_rewrite(&mut self, _replacement: RcDagNode) {
    // Empty default implementation
  }

  fn trace_pre_sc_application(&mut self, _subject: RcDagNode, _sc: &SortConstraint) {
    // Empty default implementation
  }

  fn trace_abort(&mut self) -> bool {
    false
  }

  fn trace_begin_fragment(
    &mut self,
    _trial_ref     : &mut Option<i32>,
    _pre_equation  : &dyn ConditionFragment,
    _first_attempt : bool,
  ) {
    // Empty default implementation
  }

  fn trace_end_fragment(
    &mut self,
    _trial_ref     : &mut Option<i32>,
    _pre_equation  : &dyn ConditionFragment,
    _success       : bool,
  ) {
    // Empty default implementation
  }

  fn trace_narrowing_step(
    &mut self,
    _rule         : &Rule,
    _redex        : RcDagNode,
    _replacement  : RcDagNode,
    _variable_info: &NarrowingVariableInfo,
    _substitution : &Substitution,
    _new_state    : RcDagNode,
  ) {
    // Empty default implementation
  }

  fn trace_variant_narrowing_step(
    &mut self,
    _equation                : &Equation,
    _old_variant_substitution: &Vec<RcDagNode>,
    _redex                   : RcDagNode,
    _replacement             : RcDagNode,
    _variable_info           : &NarrowingVariableInfo,
    _substitution            : &Substitution,
    _new_state               : RcDagNode,
    _new_variant_substitution: &Vec<RcDagNode>,
    _original_variables      : &NarrowingVariableInfo,
  ) {
    // Empty default implementation
  }

  fn trace_strategy_call(
    &mut self,
    _strategy_definition: &StrategyDefinition,
    _call_dag: RcDagNode,
    _subject: RcDagNode,
    _substitution: &Substitution
  ){
    // Empty default implementation
  }

  // endregion

  // Interrupt handling… not yet implemented.

  // Garbage collection… Using reference counting (std::rc::Rc) for now.

  // region Rebuilding Stale DagNodes

  fn rebuild_upto_root(&mut self) {
    let mut i: i32;
    let mut current_idx: i32;
    { // Scope of members
      let members = self.members_mut();
      // println!("\nroot was {:?}", self.root_node);
      // println!("rebuilding from {}", self.current_index);
      assert!(members.current_index >= 0, "bad currentIndex");

      // Locate deepest stack node with a stale parent.
      current_idx = members.current_index; // all staleness guaranteed to be above currentIndex
      while members.redex_stack[current_idx as usize].parent_index != members.stale_marker {
        current_idx = members.redex_stack[current_idx as usize].parent_index;
      }

      // We assume that we only have to rebuild the spine from staleMarker to root.
      i = members.stale_marker;
    } // Release mutable borrow of members.

    while i != UNDEFINED {
      self.remake_stale_dag_node(i, current_idx);
      current_idx = i;
      i = self.members()
              .redex_stack[i as usize]
              .parent_index;
    }

    // Need the mutable borrow of members again.
    let members = self.members_mut();
    members.root_node = members.redex_stack[0].dag_node.clone();
    members.stale_marker = ROOT_OK;

    // println!("root is {:?}", self.root_node);
  }

  fn remake_stale_dag_node(&mut self, stale_index: i32, child_index: i32) {
    // Find first stacked argument of stale dag node.
    let mut first_idx = child_index as usize;
    let members = self.members_mut();
    while members.redex_stack[first_idx - 1].parent_index == stale_index {
      first_idx -= 1;
    }

    // Find last stacked argument of stale dag node.
    let mut last_idx = child_index as usize;
    let stack_length = members.redex_stack.len();
    while last_idx + 1 < stack_length && members.redex_stack[last_idx + 1].parent_index == stale_index {
      last_idx += 1;
    }

    // Replace stale dag node with a copy in which stacked arguments
    // replace corresponding arguments in the original.
    let remade = members.redex_stack[stale_index as usize]
        .dag_node.borrow()
        .copy_with_replacements(&members.redex_stack, first_idx, last_idx);
    members.redex_stack[stale_index as usize].dag_node = remade;
  }


  // endregion


}
