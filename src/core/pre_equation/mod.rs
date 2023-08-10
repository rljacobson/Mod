/*!


There are multiple widgets that use the algorithmic machinery: equations, rules, patterns, sort constraints... The
`PreEquation` enum abstracts over these different widgets and provides shared implementation. Widget-specific
implementation can be found in the respective modules `equation`, `rule`, etc. For functions common to all widgets
but for which implementations differ, the method on the enum does dynamic dispatch to the implementation in the
appropriate module.

  * Equation
  * Rule
  * Membership Axiom == SortConstraint - Not yet implemented
  * StrategyDefinition (Strategy Language) - Unimplemented

ToDo: This needs a better name than `PreEquation`. Comparator? MatchClient?

*/

mod equation;
mod rule;
mod attributes;
mod sort_constraint;
pub mod sort_constraint_table;

use std::{
  rc::Rc,
  fmt::{Debug, Formatter}
};

use crate::{
  abstractions::{
    IString,
    NatSet
  },
  core::{
    condition_fragment::{
      Condition,
      repr_condition
    },
    format::{
      FormatStyle,
      Formattable
    },
    module::{ModuleItem, WeakModule},
    rewrite_context::{
      trace::trace_status,
      RewritingContext
    },
    sort::RcSort,
    StateTransitionGraph,
    substitution::Substitution,
    TermBag,
    VariableInfo,
  },
  theory::{
    DagNode,
    find_available_terms,
    index_variables,
    LHSAutomaton,
    RcDagNode,
    RcLHSAutomaton,
    RcTerm,
    Subproblem,
  },
  UNDEFINED,
};

pub use attributes::{
  PreEquationAttribute,
  PreEquationAttributes
};

pub type RcPreEquation = RcCell<PreEquation>;

/// Holds state information used in solving condition fragments.
pub enum ConditionState {
  Assignment{
    saved      : Substitution,
    rhs_context: Box<RewritingContext>,
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

/// Representation of Rule, Equation, Sort Constraint/Membership Axiom.
pub enum PreEquationKind {
  Equation{
    rhs_term           : RcTerm,
    rhs_builder        : RHSBuilder,
    fast_variable_count: i32,
    // instruction_seq    : Option<InstructionSequence>
  },

  Rule {
    rhs_term                   : RcTerm,
    rhs_builder                : RHSBuilder,
    non_extension_lhs_automaton: Option<RcLHSAutomaton>,
    extension_lhs_automaton    : Option<RcLHSAutomaton>,
  },

  SortConstraint {
    sort: RcSort
  },

  StrategyDefinition {
    // Unimplemented
  }
}
impl Debug for PreEquationKind {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Equation { .. } => {write!(f, "Equation{{…}}")}
      Rule { .. } => {write!(f, "Rule{{…}}")}
      SortConstraint { .. } => {write!(f, "SortConstraint{{…}}")}
      StrategyDefinition { .. } => {write!(f, "StrategyDefinition{{…}}")}
    }
  }
}


pub use PreEquationKind::*;
use crate::abstractions::RcCell;
use crate::core::automata::RHSBuilder;
use crate::core::interpreter::InterpreterAttribute;

impl PreEquationKind {
  pub fn noun(&self) -> &'static str {
    match self {
      Equation { .. }           => "equation",
      Rule { .. }               => "rule",
      SortConstraint { .. }     => "sort constraint",
      StrategyDefinition { .. } => "strategy definition",
    }
  }

  pub fn interpreter_trace_attribute(&self) -> InterpreterAttribute {
    match &self {
      Equation { .. }           => InterpreterAttribute::TraceEq,
      Rule { .. }               => InterpreterAttribute::TraceRl,
      SortConstraint { .. }     => InterpreterAttribute::TraceMb,
      StrategyDefinition { .. } => InterpreterAttribute::TraceSd,
    }
  }
}

pub struct PreEquation {
  pub(crate) name         : Option<IString>,
  attributes   : PreEquationAttributes,
  pub(crate) lhs_term     : RcTerm,
  lhs_automaton: Option<RcLHSAutomaton>,
  lhs_dag      : Option<RcDagNode>,
  condition    : Condition,
  pub(crate) variable_info: VariableInfo,

  // `ModuleItem`
  index_within_parent_module: i32,
  pub(crate) parent_module             : WeakModule,

  pub(crate) kind: PreEquationKind
}

impl PreEquation {
  // Common implementation
  fn trace_begin_trial(&self, subject: RcDagNode, context: &mut RewritingContext) -> Option<i32>{
    context.trace_begin_trial(subject, self)
  }

  // region Accessors
  #[inline(always)]
  pub(crate) fn condition(&self) -> &Condition {
    &self.condition
  }
  /*
  #[inline(always)]
  fn lhs_term(&self) -> RcTerm{
    self.lhs_term.clone()
  }
  #[inline(always)]
  fn lhs_automaton(&self) -> RcLHSAutomaton{
    self.lhs_automaton.as_ref().unwrap().clone()
  }
  #[inline(always)]
  fn lhs_dag(&self) -> RcDagNode{
    self.lhs_dag.as_ref().unwrap().clone()
  }
  #[inline(always)]
  fn condition_mut(&mut self) -> &mut Condition {
    &mut self.condition
  }
  #[inline(always)]
  pub(crate) fn variable_info(&self) -> &VariableInfo{
    &self.variable_info
  }
  #[inline(always)]
  fn variable_info_mut(&mut self) -> &mut VariableInfo{
    &mut self.variable_info
  }
  #[inline(always)]
  fn name(&self) -> Option<IString> {
    self.name.clone()
  }
  */ //endregion

  // region  Attributes
  #[inline(always)]
  pub(crate) fn has_condition(&self) -> bool{
    // ToDo: Can we not just check for empty?
    self.condition.is_empty()
  }
  #[inline(always)]
  fn is_nonexec(&self) -> bool {
    self.attribute(PreEquationAttribute::NonExecute)
  }
  #[inline(always)]
  fn is_compiled(&self) -> bool{
    self.attribute(PreEquationAttribute::Compiled)
  }
  #[inline(always)]
  fn is_variant(&self) -> bool{
    self.attribute(PreEquationAttribute::Variant)
  }
  #[inline(always)]
  fn set_nonexec(&mut self) {
    self.attributes |= PreEquationAttribute::NonExecute;
  }
  #[inline(always)]
  fn set_variant(&mut self) {
    self.attributes |= PreEquationAttribute::Variant;
  }
  #[inline(always)]
  pub fn is_narrowing(&self) -> bool {
    self.attribute(PreEquationAttribute::Narrowing)
  }
  #[inline(always)]
  fn attribute(&self, attribute: PreEquationAttribute) -> bool {
    self.attributes.has_attribute(attribute)
  }
  // endregion

  // region Check* functions

  /// Normalize lhs and recursively collect the indices and occurs sets of this term and its descendants
  fn check(&mut self) {
    self.lhs_term.borrow_mut().normalize(true);
    index_variables(self.lhs_term.clone(), &mut self.variable_info);

    let mut bound_variables: NatSet = self.lhs_term.borrow().occurs_below().clone(); // Deep copy

    for i in 0..self.condition.len() {
      let condition_fragment = self.condition[i].clone();
      condition_fragment.borrow_mut().check(&mut self.variable_info, &mut bound_variables);
    }

    match &self.kind {
      Equation { .. } => {
        equation::check(self, bound_variables);
      }
      Rule { .. } => {
        rule::check(self, bound_variables);
      }
      SortConstraint { .. } => {
        // Doesn't use bound_variables.
        sort_constraint::check(self);
      }
      StrategyDefinition { .. } => {
        unimplemented!()
      }
    }
  }


  ///  This is the most general condition checking function that allows multiple distinct successes; caller must provide
  ///  trial_ref variable and condition state stack in order to preserve this information between calls.
  fn check_condition(
    &mut self,
    mut find_first: bool,
    subject       : RcDagNode,
    context       : &mut RewritingContext,
    mut subproblem: Option<&mut dyn Subproblem>,
    trial_ref     : &mut Option<i32>,
    state         : &mut Vec<ConditionState>,
  ) -> bool
  {
    assert_ne!(self.condition.len(), 0, "no condition");
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

        context.trace_end_trial(*trial_ref, success);
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
      context.trace_exhausted(*trial_ref);
    }
    false
  }

  /// Simplified interface to `check_condition(…)` for the common case where we only care
  /// if a condition succeeds at least once or fails.
  fn check_condition_simple(
    &mut self,
    subject: RcDagNode,
    context: &mut RewritingContext,
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

  fn compile(&mut self, compile_lhs: bool) {
    match self.kind {

      Equation { .. }           => {
        equation::compile(self, compile_lhs);
      }

      Rule { .. }               => {
        rule::compile(self, compile_lhs);
      }

      SortConstraint { .. }     => {
        sort_constraint::compile(self, compile_lhs);
      }

      StrategyDefinition { .. } => {
        unimplemented!()
      }

    }
  }

  fn compile_build(&mut self, available_terms: &mut TermBag, eager_context: bool) {
    // Fill the hash set of terms for structural sharing
    find_available_terms(self.lhs_term.clone(), available_terms, eager_context, true);
    {// Scope of `lhs_term`
      let mut lhs_term = self.lhs_term.borrow_mut();
      lhs_term.determine_context_variables();
      lhs_term.insert_abstraction_variables(&mut self.variable_info);
    }

    let fragment_count = self.condition.len();
    for i in 0..fragment_count {
      let condition_fragment = &self.condition[i].clone();
      let mut condition_fragment = condition_fragment.borrow_mut();
      condition_fragment.compile_build(&mut self.variable_info, available_terms);
    }
  }

  fn compile_match(&mut self, compile_lhs: bool, with_extension: bool) {
    let mut lhs_term = self.lhs_term.borrow_mut();

    let index_remapping = self.variable_info.compute_index_remapping();
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

      let lhs_automaton =
          lhs_term.compile_lhs(
                with_extension,
                &mut self.variable_info,
                &mut bound_uniquely,
              )
              .0; // Disregard `subproblem_likely` component of returned tuple.
      self.lhs_automaton = Some(lhs_automaton);
    }

    { // Scope of variable_info
      let fragment_count = self.condition.len();
      for i in 0..fragment_count {
        let fragment = &self.condition[i].clone();
        fragment.borrow_mut().compile_match(&mut self.variable_info, lhs_term.occurs_below_mut());
      }
    }
  }

  // endregion

  fn solve_condition(
    &mut self,
    mut find_first: bool,
    trial_ref: &mut Option<i32>,
    solution: &mut RewritingContext,
    state: &mut Vec<ConditionState>,
  ) -> bool
  {
    let fragment_count = self.condition.len();
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
          *trial_ref,
          self.condition[i].as_ref(),
          find_first
        );
      }

      // A cute way to do backtracking.
      find_first = self.condition[i].borrow_mut().solve(find_first, solution, state);

      if trace_status() {
        if solution.trace_abort() {
          return false;
        }
        solution.trace_end_fragment(
          *trial_ref,
          self, //.condition[i].as_ref(),
          i,
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


  fn reset(&mut self) {
    self.lhs_dag = None;
  }

}

impl ModuleItem for PreEquation {
  fn get_index_within_module(&self) -> i32 {
    self.index_within_parent_module
  }

  fn set_module_information(&mut self, module: WeakModule, index_within_module: i32) {
    self.parent_module = module;
    self.index_within_parent_module = index_within_module;
  }

  fn get_module(&self) -> WeakModule {
    self.parent_module.clone()
  }
}

impl Formattable for PreEquation {
  fn repr(&self, style: FormatStyle) -> String {
    let mut accumulator = String::new();

    if style != FormatStyle::Simple {
      if self.has_condition() {
        accumulator.push('c');
      }
      match self.kind {
        Equation { .. } => {
          accumulator.push_str("eq ");
        }
        Rule { .. } => {
          accumulator.push_str("rl ");
        }
        SortConstraint { .. } => {
          accumulator.push_str("mb ");
        }
        StrategyDefinition { .. } => {
          accumulator.push_str("sd ");
        }
      }
    }


    match &self.kind {
      Equation {rhs_term, .. } => {
        accumulator.push_str(
          format!(
            "{} = {}",
            self.lhs_term.borrow().repr(style),
            rhs_term.borrow().repr(style)
          ).as_str()
        );
      }
      Rule {rhs_term, .. } => {
        accumulator.push_str(
          format!(
            "{} => {}",
            self.lhs_term.borrow(),
            rhs_term.borrow()
          ).as_str()
        );
      }
      SortConstraint {sort, .. } => {
        accumulator.push_str(
          format!(
            "{} : {}",
            self.lhs_term.borrow(),
            sort.borrow()
          ).as_str()
        );
      }
      StrategyDefinition { .. } => {
        unimplemented!("Strategy definitions not supported")
      }
    }

    if self.has_condition() {
      accumulator.push(' ');
      repr_condition(&self.condition, style);
    }

    { // Scope of attributes
      if !self.attributes.is_empty() {
        accumulator.push_str(self.attributes.repr(style).as_str());
      }
    }

    if style != FormatStyle::Simple {
      accumulator.push_str(" .");
    }

    accumulator
  }

}

