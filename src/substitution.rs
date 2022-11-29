/*!

A `Substitution` is a thin wrapper around a `Vec<&DagNode>`. It holds bindings between natural numbers and `DagNode`s
by placing a reference to the DagNode at the index of the number. Names are numbers, so these are bindings of names.

 */



use crate::theory::DagNode;

pub struct Substitution<'a> {
  bindings: Vec<&'a dyn DagNode>,
}

impl<'a> Substitution<'a> {
  pub fn new() -> Self {
    Self { bindings: Vec::new() }
  }

  pub fn with_capacity(n: usize) -> Self {
    Self {
      bindings: Vec::with_capacity(n),
    }
  }

  pub fn get(&self, index: usize) -> Option<&dyn DagNode> {
    self.bindings.get(index)
  }

  pub fn get_mut(&mut self, index: usize) -> Option<&mut dyn DagNode> {
    self.bindings.get_mut(index)
  }


}
