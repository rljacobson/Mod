/*!


*/



use std::iter::FilterMap;
use std::slice::Iter;
use crate::core::substitution::MaybeDagNode;
use crate::theory::RcDagNode;


pub struct NarrowingVariableInfo {
  variables: Vec<MaybeDagNode>
}

impl NarrowingVariableInfo {
  #[inline(always)]
  pub(crate) fn variable_count(&self) -> usize {
    self.variables.len()
  }

  #[inline(always)]
  pub(crate) fn index2variable(&self, index: usize) -> MaybeDagNode {
    if index < self.variables.len() {
      self.variables[index].clone()
    } else {
      None
    }
  }

  pub(crate) fn variable_to_index(&mut self, variable: RcDagNode) -> i32 {
    // assert!(variable != &VariableTerm::default(), "null term");
    let idx = self.variables
        .iter()
        .position(
          |v| {
            v.is_some()
                && v.unwrap()
                .borrow()
                .compare(&*variable.borrow()).is_eq()
          }
        );
    match idx {
      Some(i) => i as i32,
      None => {
        self.variables.push(Some(variable.clone()));
        (self.variables.len() - 1) as i32
      }
    }

  }

  pub(crate) fn iter(&self) -> Box<dyn Iterator<Item=(usize, RcDagNode)>> {
    Box::new(self.variables.iter().filter_map(|v| (*v).clone()).enumerate())
  }

  pub(crate) fn variable_to_index_without_insert(&mut self, variable: RcDagNode) -> Option<i32> {
    // assert!(variable != &VariableTerm::default(), "null term");
    self.variables
        .iter()
        .position(
          |v| {
            v.is_some()
                && v.unwrap()
                    .borrow()
                    .compare(&*variable.borrow()).is_eq()
          }
        )
        .map(|i| i as i32)
  }

}
