/*!

Trait for DAG nodes.

 */

use std::ops::BitAnd;
use std::any::Any;
use std::slice::Iter;

use crate::theory::symbol::Symbol;

pub struct DagPair {
  dag_node    : Box<dyn DagNode>,
  multiplicity: u32
}

// todo: Maude puts `copyPointer` and `top_symbol` in a union for optimization.
pub(crate) trait DagNode {
  fn top_symbol(&self) -> &Symbol;
  fn top_symbol_mut(&mut self) -> &mut Symbol;

  fn args(&self) -> Iter<DagPair>;

  fn as_any(&self) -> &dyn Any;
}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
