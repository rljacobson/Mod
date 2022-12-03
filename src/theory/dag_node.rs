/*!

Trait for DAG nodes.

 */

use std::any::Any;
use std::cell::Cell;
use std::cmp::Ordering;

use dyn_clone::{clone_trait_object, DynClone};

use crate::theory::symbol::Symbol;

// pub type BcDagNode = Box<Cell<DagNode>>;
pub type BcDagNode = Box<dyn DagNode>;

/// This struct owns the DagNode. If we just want a reference, we use a tuple `(dag_node.as_ref(), multiplicity)`.
#[derive(Clone)]
pub struct DagPair {
  pub(crate) dag_node    : BcDagNode,
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

impl Eq for dyn DagNode {}

impl PartialEq<Self> for dyn DagNode {
  fn eq(&self, other: &Self) -> bool {
    self.symbol().eq(other.symbol())
  }
}


impl PartialOrd<Self> for dyn DagNode {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for dyn DagNode {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.symbol().cmp(&other.symbol())
  }
}
