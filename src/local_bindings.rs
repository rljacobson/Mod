/*!

Implements a map between variables and values.

The implementation is not optimized for speed of lookup. It is just a list of records. Indeed, no deduplication or
other validation is performed.

*/


use crate::theory::{DagNode, RcDagNode};

#[derive(Eq, PartialEq, Debug)]
pub struct Binding {
  active        : bool,
  variable_index: u32,
  value         : RcDagNode,
}

#[derive(Default)]
pub struct LocalBindings {
  pub bindings: Vec<Binding>
}

impl LocalBindings {
  pub fn new() -> LocalBindings {
    Self::default()
  }

  pub fn add_binding(&mut self, index: u32, value: RcDagNode) {
    self.bindings.push(
      Binding{
        active: false,
        variable_index: index,
        value
      }
    );
  }

  pub fn len(&self) -> usize {
    self.bindings.len()
  }
}
