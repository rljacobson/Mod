/*!

A kind of "dummy" RHS automata used, for example, to just do the substitution.

*/


use std::any::Any;
use crate::core::substitution::{MaybeDagNode, Substitution};
use crate::core::VariableInfo;
use crate::theory::{RcDagNode, RHSAutomaton};

#[derive(Copy, Clone, Default)]
pub(crate) struct TrivialRHSAutomaton {
  index: i32
}

impl TrivialRHSAutomaton {
  pub fn new(index: i32) -> Self {
    Self {
      index
    }
  }
}

impl RHSAutomaton for TrivialRHSAutomaton {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    self.index = variable_info.remap_index(self.index );
  }

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    return matcher.value(self.index as usize);
  }

  fn replace(&mut self, old: RcDagNode, matcher: &mut Substitution) {
    matcher.value(self.index as usize).unwrap().borrow_mut().overwrite_with_clone(old);
  }
}
