/*!



 */
use std::borrow::Borrow;
use std::cmp::Ordering;

use intrusive_collections::RBTreeLink;

use crate::theory::dag_node::{RcDagNode, DagNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RedBlackNodeFlags {
  Color = 0,

  Marked = 6,
  Collect = 7,
}

#[derive(Clone)]
pub struct RedBlackNode {
  pub dag_node        : RcDagNode,
  pub multiplicity    : u32,
  // pub max_multiplicity: u32,
  pub link            : RBTreeLink,
  pub flags            : u8
}

impl RedBlackNode {

  pub fn new(dag_node: RcDagNode, multiplicity: u32) -> Self {
    RedBlackNode{
      dag_node,
      multiplicity,
      // max_multiplicity: 0,
      link: RBTreeLink::default(),
      flags: 0
    }
  }
/*
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
  */
}

impl Eq for RedBlackNode{}

impl PartialEq<Self> for RedBlackNode {
  fn eq(&self, other: &Self) -> bool {
    self.dag_node.as_ref().borrow().eq(other.dag_node.as_ref())
  }
}

impl PartialOrd for RedBlackNode {
  fn partial_cmp(&self, other: &RedBlackNode) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for RedBlackNode {
  fn cmp(&self, other: &Self) -> Ordering {
    self.dag_node.as_ref().borrow().cmp(other.dag_node.as_ref())
  }
}

