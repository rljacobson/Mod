/*!



 */


use crate::theory::dag_node::{DagNode, DagPair};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RedBlackNodeFlags {
  Color = 0,

  Marked = 6,
  Collect = 7,
}

#[derive(Clone)]
pub struct RedBlackNode {
  pub dag_node        : Box<dyn DagNode>,
  pub multiplicity    : u32,
  pub max_multiplicity: u32,
  pub left            : Option<Box<RedBlackNode>>,
  pub right           : Option<Box<RedBlackNode>>,
  pub flags           : u8
}

impl RedBlackNode {
  pub fn get_child(&self, sign: u32) -> Option<&Box<RedBlackNode>> {
    if sign < 0 {
      if let Some(&left) = self.left {
        return Some(left);
      }
    } else {
      if let Some(&right) = self.right {
        return Some(right);
      }
    }

    return None;
  }

  /// Moves all `DagNode`s onto the `nodes` vector in order, consuming self.
  pub fn vectorize(self, nodes: &mut Vec<DagPair>) {
    if let Some(left) = self.left {
      left.vectorize(nodes);
    }
    let dag_pair = DagPair{
      dag_node: self.dag_node,
      multiplicity: self.multiplicity
    };
    nodes.push(dag_pair);
    if let Some(right) = self.right {
      right.vectorize(nodes);
    }
  }
}



#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
