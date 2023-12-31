/*!

This wraps a `HashSet<RcDagNode>` so that operations specific to canonicity are built in.

*/

use crate::{
  abstractions::{HashSet, HashValueType, RcCell},
  theory::{DagNode, RcDagNode},
};

pub struct HashConsSet {
  inner: HashSet<RcDagNode>,
}

impl Default for HashConsSet {
  fn default() -> Self {
    HashConsSet {
      inner: HashSet::default(),
    }
  }
}

impl HashConsSet {
  /// If a version of the node is already in the bag, upgrade its sort.
  /// Otherwise, insert a _canonical copy_ of the node into the bag.
  pub fn insert(&mut self, node: RcDagNode) -> (RcDagNode, HashValueType) {
    match self.inner.find(&node) {
      Some((existing_node, existing_node_hash)) => {
        // Found an existing node.
        existing_node
          .borrow_mut()
          .upgrade_sort_index(node.borrow().sort_index());

        // Return the existing node and its hash value.
        (existing_node.clone(), existing_node_hash)
      }
      None => {
        // Node does not exist, insert it after making a canonical copy.
        let canonical_node = node.borrow().make_canonical(node.clone(), self);
        let hash = self.hash_value(&canonical_node);

        // Insert the canonical node into the set.
        self.inner.insert(canonical_node.clone());

        // Return the new node and its hash value.
        (canonical_node, hash)
      }
    }
  }

  ///  The only difference form the above function is we never insert the original; if there is no existing copy and
  ///  we're not forced to make a copy because of non-canonical arguments, we make a copy anyway. This is useful where
  ///  the original may be reduced in place and is therefore not safe to put in a hash cons table.
  ///
  ///  We make an assumption here that that any sort in d is either unknown or unimportant.
  pub fn insert_copy(&mut self, node: RcDagNode) -> (RcDagNode, HashValueType) {
    // Always create a canonical copy of the node, regardless of whether an equivalent node exists.
    let (canonical_node, hash) = self.make_canonical(node);
    // let hash = self.hash_value(&canonical_node);

    // Insert the canonical node into the set. Unlike `insert`, do this regardless of whether
    // a node with the same value already exists, as we always want a copy in this case.
    self.inner.insert(canonical_node.clone());

    // Return the new node and its hash value.
    (canonical_node, hash)
  }
}
