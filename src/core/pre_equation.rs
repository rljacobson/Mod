/*!

There are multiple widgets that use the algorithmic machinery: equations, rules, patterns, sort constraints... This
trait abstracts over these different widgets and provides shared implementation.

ToDo: This needs a better name than `PreEquation`. Comparator? MatchClient?

*/


use std::cell::RefCell;
use crate::abstractions::{IString, NatSet};
use crate::core::{RewritingContext, ConditionFragment, RcConditionFragment, Condition, VariableInfo, Substitution, StateTransitionGraph, TermBag};
pub use crate::core::pre_equation_attributes::{PreEquationAttributes, PreEquationAttribute};
use crate::core::rewrite_context::trace_status;
use crate::theory::{DagNode, LHSAutomaton, RcDagNode, RcLHSAutomaton, RcTerm, Subproblem, index_variables, find_available_terms};
use crate::UNDEFINED;


/// Holds state information used in solving condition fragments.
pub enum ConditionState {
  Assignment{
    saved      : Substitution,
    rhs_context: Box<dyn RewritingContext>,
    subproblem : Box<dyn Subproblem>,
    succeeded  : bool
  },

  Rewrite{
    state_graph: StateTransitionGraph,
    matcher    : Box<dyn LHSAutomaton>,
    saved      : Substitution,
    subproblem : Box<dyn Subproblem>,
    explore    : i32,
    edge_count : u32
  }
}


pub(crate) struct PreEquationMembers {
  name         : IString,
  attributes   : PreEquationAttributes,
  lhs_term     : RcTerm,
  lhs_automaton: RcLHSAutomaton,
  lhs_dag      : RcDagNode, // ToDo: Why not just fetch it from the `lhs_term`? (Maude: "for unification")
  condition    : Condition,
  variable_info: VariableInfo,
}

pub(crate) trait PreEquation {
  // Common implementation
  fn members_mut(&mut self) -> &mut PreEquationMembers;
  fn members(&self) -> &PreEquationMembers;
  /// This one is a bit odd. The idea is that a `RewritingContext` supports multiple kinds of `trace_begin_trial`-like
  /// calls, and only the implementor of `PreEquation` knows which to call.
  // ToDo: This is just a bad design. The "different" receivers are virtually identical. Refactor this.
  fn trace_begin_trial(&self, subject: RcDagNode, context: &mut dyn RewritingContext) -> Option<i32>;

  // region Accessors
  #[inline(always)]
  fn lhs_term(&self)      -> RcTerm{
    self.members().lhs_term.clone()
  }
  #[inline(always)]
  fn lhs_automaton(&self) -> RcLHSAutomaton{
    self.members().lhs_automaton.clone()
  }
  #[inline(always)]
  fn lhs_dag(&self)   -> RcDagNode{
    self.members().lhs_dag.clone()
  }
  #[inline(always)]
  fn condition_mut(&mut self) -> &mut Condition {
    &mut self.members_mut().condition
  }
  #[inline(always)]
  fn condition(&self) -> &Condition {
    &self.members().condition
  }
  #[inline(always)]
  fn has_condition(&self) -> bool{
    // ToDo: Can we not just check for empty?
    self.members().condition.is_empty()
  }
  #[inline(always)]
  fn variable_info(&self) -> &VariableInfo{
    &self.members().variable_info
  }
  #[inline(always)]
  fn variable_info_mut(&mut self) -> &mut VariableInfo{
    &mut self.members_mut().variable_info
  }
  // endregion

  // Attributes
  fn is_nonexec(&self) -> bool;
  fn is_compiled(&self) -> bool;
  fn set_nonexec(&mut self);

  // region Check* functions

  /// Normalize lhs and recursively collect the indices and occurs sets of this term and its descendants
  fn check(&mut self) -> NatSet{
    self.lhs_term().borrow_mut().normalize(true);
    index_variables(self.lhs_term().clone(), self.variable_info_mut());

    let mut bound_variables: NatSet = self.lhs_term().borrow().occurs_below().clone(); // Deep copy

    for i in 0..self.condition().len() {
      let condition_fragment = self.condition()[i].clone();
      condition_fragment.borrow_mut().check(self.variable_info_mut(), &mut bound_variables);
    }

    bound_variables
  }


  ///  This is the most general condition checking function that allows multiple distinct successes; caller must provide
  ///  trial_ref variable and condition state stack in order to preserve this information between calls.
  fn check_condition(
    &mut self,
    mut find_first: bool,
    subject: RcDagNode,
    context: &mut dyn RewritingContext,
    mut subproblem: Option<&mut dyn Subproblem>,
    trial_ref: &mut Option<i32>,
    state: &mut Vec<ConditionState>,
  ) -> bool {
    assert_ne!(self.condition().len(), 0, "no condition");
    assert!(!find_first || state.is_empty(), "non-empty condition state stack");

    if find_first {
      *trial_ref = None;
    }

    loop {
      if trace_status() {
        if find_first {
          *trial_ref = self.trace_begin_trial(subject.clone(), context);
        }
        if context.trace_abort() {
          state.clear();
          // return false since condition variables may be unbound
          return false;
        }
      }

      let success: bool = self.solve_condition(find_first, trial_ref, context, state);

      if trace_status() {
        if context.trace_abort() {
          state.clear();
          return false; // return false since condition variables may be unbound
        }

        context.trace_end_trial(trial_ref, success);
      }

      if success {
        return true;
      }
      assert!(state.is_empty(), "non-empty condition state stack");
      find_first = true;
      *trial_ref = None;

      // Condition evaluation may create nodes without doing rewrites so run GC safe point.
      // MemoryCell::ok_to_collect_garbage();
      if let Some(subproblem) = &mut subproblem {
        if !subproblem.solve(false, context) {
          break;
        }
      } else {
        break;
      }
    }
    if trace_status() && trial_ref.is_some() {
      context.trace_exhausted(trial_ref);
    }
    false
  }

  /// Simplified interface to `check_condition(…)` for the common case where we only care
  /// if a condition succeeds at least once or fails.
  fn check_condition_simple(
    &mut self,
    subject: RcDagNode,
    context: &mut dyn RewritingContext,
    subproblem: Option<&mut dyn Subproblem>,
  ) -> bool
  {
    let mut trial_ref: Option<i32> = None;
    let mut state: Vec<ConditionState> = Vec::new();

    let result = self.check_condition(true, subject, context, subproblem, &mut trial_ref, &mut state);

    assert!(result || state.is_empty(), "non-empty condition state stack");
    // state drops its elements when it goes out of scope.
    // state.clear();

    result
  }

  // endregion

  // region Compile Functions

  fn compile_build(&mut self, available_terms: &mut TermBag, eager_context: bool) {
    // Fill the hash set of terms for structural sharing
    find_available_terms(self.lhs_term().clone(), available_terms, eager_context, true);
    {// Scope of `variable_info` and `lhs_term`
      let lhs_term = self.lhs_term();
      let mut lhs_term = lhs_term.borrow_mut();
      lhs_term.determine_context_variables();

      let variable_info = self.variable_info_mut();
      lhs_term.insert_abstraction_variables(variable_info);
    }

    let fragment_count = self.condition().len();
    for i in 0..fragment_count {
      let condition_fragment = self.condition()[i].clone();
      let mut condition_fragment = condition_fragment.borrow_mut();
      condition_fragment.compile_build(self.variable_info_mut(), available_terms);
    }
  }

  fn compile_match(&mut self, compile_lhs: bool, with_extension: bool) {
    let lhs_term = self.lhs_term();
    let mut lhs_term = lhs_term.borrow_mut();

    let index_remapping =
        { // Scope of variable_info
          let variable_info = self.variable_info_mut();
          variable_info.compute_index_remapping()
        };
    // We don't assume that our module was set, so we look at the module of the lhs top symbol.
    // This is the craziest pointer chasing I have ever seen.
    lhs_term.symbol()
            .as_ref()
            .symbol_members()
            .parent_module
            .upgrade()
            .unwrap()
            .borrow_mut()
            .notify_substitution_size(index_remapping);


    if compile_lhs {
      let mut bound_uniquely = NatSet::new();

      let variable_info = self.variable_info_mut();
      self.members_mut().lhs_automaton =
          lhs_term.compile_lhs(
                with_extension,
                variable_info,
                &mut bound_uniquely,
              )
              .0; // Disregard `subproblem_likely` component of returned tuple.
    }

    { // Scope of variable_info
      let fragment_count = self.condition().len();
      for i in 0..fragment_count {
        let fragment = self.condition()[i].clone();
        let variable_info = self.variable_info_mut();
        fragment.borrow_mut().compile_match(variable_info, lhs_term.occurs_below_mut());
      }
    }
  }

  // endregion

  fn solve_condition(
    &mut self,
    mut find_first: bool,
    trial_ref: &mut Option<i32>,
    solution: &mut dyn RewritingContext,
    state: &mut Vec<ConditionState>,
  ) -> bool
  {
    let condition = self.condition_mut();
    let fragment_count = condition.len();
    let mut i = if find_first {
          0
        } else {
          fragment_count - 1
        };

    loop {
      if trace_status() {
        if solution.trace_abort() {
          return false;
        }
        solution.trace_begin_fragment(
          trial_ref,
          condition[i].as_ref(),
          find_first
        );
      }

      // A cute way to do backtracking.
      find_first = condition[i].borrow_mut().solve(find_first, solution, state);

      if trace_status() {
        if solution.trace_abort() {
          return false;
        }
        solution.trace_end_fragment(
          trial_ref,
          condition[i].as_ref(),
          find_first
        );
      }

      if find_first {
        if i == fragment_count - 1 {
          break;
        }
        i += 1;
      } else {
        if i == 0 {
          break;
        }
        i -= 1;
      }
    }

    find_first
  }



  fn reset(&mut self);
  fn repr(&self, s: &mut dyn std::io::Write) -> std::io::Result<()>;
}
