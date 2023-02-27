/*!

Implements a map between variables and values.

The implementation is not optimized for speed of lookup. It is just a list of records. Indeed, no deduplication or
other validation is performed.

*/


use crate::{theory::RcDagNode, substitution, Substitution};

#[derive(Eq, PartialEq)]
struct Binding {
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

  pub fn assert(&self, substitution: &mut Substitution) -> bool {
    for i in self.bindings.iter() {
      if let Some(d) = substitution.value(i.variable_index){
        if d != i.value {
          return false;
        }
      }
    }

    for i in self.bindings.iter_mut() {
      let index = i.variable_index;
      if substitution.value(index).is_none() {
        substitution.bind(index, Some(i.value));
        i.active = true;
      }
    }

    true
  }

  pub fn retract(&mut self, substitution: &mut Substitution) {
    for i in self.bindings.iter_mut() {
      if i.active {
        i.active = false;
        substitution.bind(i.variable_index, None);
      }
    }
  }

}
