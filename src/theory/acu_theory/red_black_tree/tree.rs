/*!



 */

use crate::theory::term::Term;
use super::RedBlackNode;


// The point of having an ACU tree instead of a naked ACUTreeNode is so we can keep track of size in constant time.
pub struct RedBlackTree {
  root: Box<RedBlackNode>,
  size: u32,
}

impl RedBlackTree {
  /// If found, returns the path to the node for the key.
  pub fn node_for_key(&self, key: &Term) -> Option<Vec<&Box<RedBlackNode>>> {
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
}
