/*!

Some pre-equations have conditions associated with them that must be satisfied in order for the pre-equation to
apply. The condition can have multiple parts, called fragments. These fragments are themselves terms that are matched
against. So `ConditionFragment` is like a "lite" version of `PreEquation`.

ToDo: Should this and PreEquation be unified or refactored.

*/


use std::cell::RefCell;
use std::rc::Rc;
use crate::{
  core::{
    pre_equation::ConditionState,
    RewritingContext,
    VariableInfo,
    TermBag
  },
  abstractions::NatSet,
};
use crate::abstractions::RcCell;

/// A `Condition` is a set of `ConditionFragments`.
pub type Condition = Vec<RcConditionFragment>;
pub type RcConditionFragment = RcCell<dyn ConditionFragment>;


pub trait ConditionFragment {
  fn check(&self, var_info: &mut VariableInfo, bound_variables: &mut NatSet);
  fn preprocess(&mut self);
  fn compile_build(&mut self, variable_info: &mut VariableInfo, available_terms: &mut TermBag);
  fn compile_match(&mut self, variable_info: &mut VariableInfo, bound_uniquely: &mut NatSet);
  fn solve(
    &mut self,
    find_first: bool,
    solution: &mut dyn RewritingContext,
    state: &mut Vec<ConditionState>,
  ) -> bool;
}
