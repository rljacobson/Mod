/*!

Implements a map between variables and values.

The implementation is not optimized for speed of lookup. It is just a list of records. Indeed, no deduplication or
other validation is performed.

*/


use crate::{core::substitution::Substitution, theory::RcDagNode};

pub struct Binding {
  active:         bool,
  variable_index: i32,
  value:          RcDagNode,
}

#[derive(Default)]
pub struct LocalBindings {
  pub bindings: Vec<Binding>,
}

impl LocalBindings {
  pub fn new() -> LocalBindings {
    Self::default()
  }

  pub fn add_binding(&mut self, index: i32, value: RcDagNode) {
    self.bindings.push(Binding {
      active: false,
      variable_index: index,
      value,
    });
  }

  pub fn len(&self) -> usize {
    self.bindings.len()
  }

  pub fn assert(&mut self, substitution: &mut Substitution) -> bool {
    for i in self.bindings.iter() {
      if let Some(d) = substitution.get(i.variable_index) {
        if d.as_ref() != i.value.as_ref() {
          return false;
        }
      }
    }

    for i in self.bindings.iter_mut() {
      let index = i.variable_index;
      if substitution.get(index).is_none() {
        substitution.bind(index, Some(i.value.clone()));
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
