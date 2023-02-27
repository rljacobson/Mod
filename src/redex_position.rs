/*!

A `RedexPosition` holds position information of a potential redex.

*/

use crate::theory::RcDagNode;


#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RedexPositionFlags {
  Stale = 1,
  Eager = 2
}


pub struct RedexPosition {
  dag_node: RcDagNode,
  p_index: u32,
  a_index: u32,
  flags: u32,
}

