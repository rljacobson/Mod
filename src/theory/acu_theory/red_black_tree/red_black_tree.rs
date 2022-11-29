/*!



 */

use crate::{
  theory::{
    DagNode,
    Term
  }
};
use crate::theory::dag_node::DagPair;
use super::RedBlackNode;


// The point of having an ACU tree instead of a naked ACUTreeNode is so we can keep track of size in constant time.
pub struct RedBlackTree {
  root: Box<RedBlackNode>,
  size: usize, // Todo: Construction methods need to update this.
}

impl RedBlackTree {
  /// If found, returns the path to the node for the key.
  pub fn node_for_key(&self, key: &dyn Term) -> Option<Vec<&Box<RedBlackNode>>> {
    let mut path = Vec::new();
    let mut root: &RedBlackNode  = &self.root;

    loop {
      path.push(&self.root);
      let r =  key.compare_dag_node(&root.dag_node);
      if r ==  0 {
        return Some(path)
      }
      if let Some(node) = root.get_child(r){
        root = node;
        continue;
      }
      return None;
    }
  }

  /// Moves all nodes of the tree into a vector, consuming self, and returns the vector.
  pub fn vectorize(self) -> Vec<DagPair> {
    let mut vector = Vec::new();
    self.root.vectorize(&mut vector);
    vector
  }


  /// Iterates over the nodes and their multiplicities.
  pub fn iter(&self) -> impl Iterator<Item=(&dyn DagNode, u32)> {
    RedBlackTreeIterator::new(
      self.root.as_ref()
    ).map(|node| (node.dag_node.as_ref(), node.multiplicity))
  }
}


struct RedBlackTreeIterator<'a>{
  stack: Vec<&'a RedBlackNode>,
}

impl<'a> RedBlackTreeIterator<'a> {
  pub fn new(root: &'a RedBlackNode) -> Self {
    RedBlackTreeIterator {
      stack: vec![root],
    }
  }

  fn stack_leftmost_path(&mut self, mut root: &'a RedBlackNode) {
    self.stack.push(root);
    while let Some(node)  = &root.left {
      self.stack.push(node);
      root = node;
    }
  }
}

impl<'a> Iterator for RedBlackTreeIterator<'a> {
  type Item = &'a RedBlackNode;

  fn next(&mut self) -> Option<&'a RedBlackNode> {
    if let Some(top_node) = self.stack.pop().unwrap(){
      if let Some(r) = top_node.right.as_ref() {
        self.stack_leftmost_path(r);
      }
      Some(top_node)
    } else {
      None
    }

  }

}
