/*!

Trait for DAG nodes.

 */

use std::any::Any;

use dyn_clone::{clone_trait_object, DynClone};

use crate::theory::symbol::Symbol;

/// This struct owns the DagNode. If we just want a reference, we use a tuple `(dag_node.as_ref(), multiplicity)`.
#[derive(Clone)]
pub struct DagPair {
  pub(crate) dag_node    : Box<dyn DagNode>,
  pub(crate) multiplicity: u32
}

// todo: Maude puts `copyPointer` and `top_symbol` in a union for optimization.
pub trait DagNode: DynClone {
  /// Gives the top symbol of this term.
  fn symbol(&self) -> &Symbol;
  fn symbol_mut(&mut self) -> &mut Symbol;

  /// Returns an iterator over `DagPair`s for the arguments.
  fn iter_args(&self) -> Box<dyn Iterator<Item=(&dyn DagNode, u32)>> ;

  /// The number of arguments.
  fn len(&self) -> u32;

  fn as_any(&self) -> &dyn Any;
}

clone_trait_object!(DagNode);

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
