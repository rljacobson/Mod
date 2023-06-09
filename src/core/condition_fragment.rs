/*!

Some pre-equations have conditions associated with them that must be satisfied in order for the pre-equation to
apply. The condition can have multiple parts, called fragments. These fragments are themselves terms that are matched
against. So `ConditionFragment` is like a "lite" version of `PreEquation`.

ToDo: Should this and PreEquation be unified or refactored.

*/


use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;


use crate::{
  abstractions::{
    NatSet,
    RcCell,
    join_iter
  },
  core::{
    TermBag,
    VariableInfo,
    format::{FormatStyle, Formattable},
    pre_equation::ConditionState,
    rewrite_context::RewritingContext,
    sort::RcSort,
    automata::RHSBuilder
  },
  theory::{
    LHSAutomaton,
    RcLHSAutomaton,
    RcTerm
  },
};

/// A `Condition` is a set of `ConditionFragments`.
pub type Condition = Vec<RcConditionFragment>;
pub type RcConditionFragment = RcCell<ConditionFragment>;

pub enum ConditionFragment {
  Equality{
    lhs_term: RcTerm,
    rhs_term: RcTerm,
    builder: RHSBuilder,
    lhs_index: i32,
    rhs_index: i32,
  },

  SortTest{
    lhs_term: RcTerm,
    sort: RcSort,
    builder: RHSBuilder,
    lhs_index: i32,
  },

  Assignment{
    lhs_term: RcTerm,
    rhs_term: RcTerm,
    builder: RHSBuilder,
    lhs_matcher: RcLHSAutomaton,
    rhs_index: i32,
  },

  Rewrite{
    lhs_term: RcTerm,
    rhs_term: RcTerm,
    builder: RHSBuilder,
    rhs_matcher: RcLHSAutomaton,
    lhs_index: i32,
  },
}

impl ConditionFragment {
  pub fn check(&self, var_info: &mut VariableInfo, bound_variables: &mut NatSet) {

  }

  pub fn preprocess(&mut self) {

  }

  pub fn compile_build(&mut self, variable_info: &mut VariableInfo, available_terms: &mut TermBag) {

  }

  pub fn compile_match(&mut self, variable_info: &mut VariableInfo, bound_uniquely: &mut NatSet) {

  }

  pub fn solve(
    &mut self,
    find_first: bool,
    solution: &mut RewritingContext,
    state: &mut Vec<ConditionState>,
  ) -> bool
  {
    false
  }
}

impl Formattable for ConditionFragment {
  fn repr(&self, style: FormatStyle) -> String {
    match self {

      ConditionFragment::Equality{lhs_term, rhs_term, ..} => {
        format!("{} = {}", lhs_term.borrow().repr(style), rhs_term.borrow().repr(style))
      },

      ConditionFragment::SortTest{ lhs_term, sort, ..} => {
        format!("{} : {}", lhs_term.borrow().repr(style), sort.borrow())
      },

      ConditionFragment::Assignment{lhs_term, rhs_term, ..} => {
        format!("{} := {}", lhs_term.borrow().repr(style), rhs_term.borrow().repr(style))
      },

      ConditionFragment::Rewrite{lhs_term, rhs_term, ..} => {
        format!("{} => {}", lhs_term.borrow().repr(style), rhs_term.borrow().repr(style))
      }

    }
  }
}


pub fn repr_condition(condition: &Condition, style: FormatStyle) -> String {
  let mut accumulator = "if ".to_string();
  accumulator.push_str(
    join_iter(condition.iter().map(|cf| cf.borrow().repr(style)), |_| " ∧ ".to_string()).collect::<String>().as_str()
  );

  accumulator
}
