/*!

A `Substitution` is a thin wrapper around a `Vec<&DagNode>`. It holds bindings between natural numbers and `DagNode`s
by placing a reference to the DagNode at the index of the number. Names are numbers, so these are bindings of names.

 From \[Eker 2003]:

 > For efficiency, the set of variable bindings at each stage in the recursion in both simplify and build_hierarchy can be tracked by a single global array indexed by small integers representing variables.

 */


use std::rc::Rc;

use crate::local_bindings::LocalBindings;
use crate::theory::{DagNode, RcDagNode};

pub type MaybeDagNode = Option<RcDagNode>;

#[derive(Clone, Default)]
pub struct Substitution {
  bindings: Vec<MaybeDagNode>,
  // Todo: What is the purpose of copy_size?
  copy_size: u32,
}

impl Substitution {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_capacity(n: u32) -> Self {
    let mut bindings = Vec::with_capacity(n as usize);
    bindings.resize(n as usize, None);

    Self {
      bindings,
      copy_size: n,
    }
  }

  pub fn resize(&mut self, size: usize) {
    self.bindings.resize(size, None);
  }

  pub fn value(&self, index: u32)  -> MaybeDagNode {
    self.get(index as usize)
  }

  
  // Todo: Is this the best way to implement a getter? I think we did it this way so it returned a value.
  pub fn get(&self, index: usize) -> MaybeDagNode {
    if (index as usize) < self.bindings.len() {
      unsafe{
        (*self.bindings.get_unchecked(index as usize)).clone()
      }
    } else {
      None
    }
  }


  pub fn iter(&self) -> std::slice::Iter<Option<Rc<dyn DagNode>>> {
    self.bindings.iter()
  }


  pub fn fragile_binding_count(&self) -> u32 {
    self.copy_size
  }


  pub fn subtract(&self, original: &Substitution) -> Option<LocalBindings> {
    let mut local_bindings = LocalBindings::new();
    for (idx, (i, j)) in self.bindings.iter().zip(original.iter()).enumerate() {
      assert!(j.is_none() || i==j, "substitution inconsistency at index {}", idx);
      if let (Some(a), Some(b)) = (i, j) {
        if a != b {
          local_bindings.add_binding(idx as u32, (*a).clone());
        }
      }
    }

    if local_bindings.len() > 0  {
      Some(local_bindings)
    } else {
      None
    }
  }


  pub fn assert(&self, solution: &Substitution) {
    // Todo: Implement assert
  }


  pub fn retract(&self, solution: &Substitution) {
    // Todo: Implement retract
  }


  pub fn bind(&mut self, index: u32, maybe_value: Option<RcDagNode>) {
    assert!(index >= 0, "Negative index {}", index);
    assert!((index as usize) < self.bindings.len(), "Index too big {} vs {}", index, self.bindings.len());

    self.bindings[index as usize] = maybe_value;
  }

  pub fn copy_from_substitution(&mut self, original: &Substitution) {
    assert!(self.copy_size == original.copy_size);

    if self.copy_size > 0 {
      self.bindings = original.bindings.clone();
    }
  }


}
