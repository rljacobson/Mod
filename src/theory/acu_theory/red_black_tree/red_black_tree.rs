/*!

Maude uses a Red-black tree map to store some data. The key is the symbol hash value and the value is a DagPair.

*/
use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::ops::Bound;
use std::rc::Rc;
use std::collections::{BTreeSet, BTreeMap};

use reffers::rc1::Strong;

use crate::{
  Substitution,
  theory::{
    DagNode,
    Term,
    DagPair,
    dag_node::RcDagNode
  }
};
use super::RedBlackNode;


pub type RcRedBlackTree = Strong<RedBlackTree>;

pub struct RedBlackTree {
  rb_tree: BTreeMap<i32, RedBlackNode>,
  pub(crate) size: usize, // Todo: Construction methods need to update this.
}

impl RedBlackTree {

  pub fn new(root: RcDagNode, multiplicity: u32) -> Self {
    let key = root.as_ref().get_sort_index();
    let root = RedBlackNode::new(root, multiplicity);

    let mut rb_tree = BTreeMap::new();
    rb_tree.insert(key, root);

    RedBlackTree {
      rb_tree,
      size: multiplicity as usize,
    }
  }

  pub fn clear(&mut self) {
    self.rb_tree.clear();
    self.size = 0;
  }

  pub fn insert(&mut self, dag_node: RcDagNode) {
    let key = dag_node.get_sort_index();
    let node = RedBlackNode::new(dag_node, 1);
    self.rb_tree.insert(key, node);
  }

  /// Gets the multiplicity of the first node in the tree. If size==1, that would be the only multiplicity in the tree.
  pub fn get_sole_multiplicity(&self) -> u32 {
    let (key, tree_node) = self.rb_tree.first_key_value().unwrap();
    *tree_node.multiplicity.borrow()
  }

  /// Gets the first node in the tree. If size==1, that would be the only node in the tree.
  pub fn get_sole_dag_node(&self) -> RcDagNode {
    let (key, tree_node) = self.rb_tree.first_key_value().unwrap();
    tree_node.dag_node.clone()
  }

  pub fn is_reduced(&self) -> bool {
    // Todo: Implement `is_reduced()` for RedBlackTree. In Maude it is a property of a DagNode, and a tree is reduced if its
    //       root is.
    true
  }

  /// Computes the maximum multiplicity of any `DagNode` occurring in the tree.
  pub fn max_multiplicity(&self) -> u32 {
    self.rb_tree
        .iter()
        .map(|(_key, node)| *node.multiplicity.borrow())
        .max()
        .unwrap_or(0)
  }

  // Todo: Why not have `find*` methods take a `&Symbol` instead of a `Term` or `DagNode`? Then we'd only need one set.

  /// If found, returns a reference to the node for the key.
  pub fn find_term(&self, key: &dyn Term) -> Option<&RedBlackNode> {
    let found: Option<&RedBlackNode> = self.rb_tree.get(&(key.symbol().get_hash_value() as i32));
    if let Some(tree_node) = found {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      if key.compare_dag_arguments(tree_node.dag_node.as_ref()) == Ordering::Equal {
        return Some(tree_node);
      }
    }

    return None;
  }

  /// If found, returns a reference to the node for the key.
  pub fn find(&self, key: &dyn DagNode) -> Option<&RedBlackNode> {
    let found: Option<&RedBlackNode> = self.rb_tree.get(&(key.symbol().get_hash_value() as i32));
    if let Some(tree_node) = found {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      if key.compare(tree_node.dag_node.as_ref()) == Ordering::Equal {
        return Some(tree_node);
      }
    }

    return None;
  }

  /// Same as above, but returns a `RedBlackNode` instead of a `&RedBlackNode`.
  pub fn find_term_mut(&self, key: &dyn Term) -> Option<&mut RedBlackNode> {
    let found: Option<&mut RedBlackNode> = self.rb_tree.get_mut(&(key.symbol().get_hash_value() as i32));
    if let Some(tree_node) = found {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      if key.compare_dag_arguments(tree_node.dag_node.as_ref()) == Ordering::Equal {
        return Some(tree_node);
      }
    }

    return None;
  }

  /// Same as above, but returns a `RedBlackNode` instead of a `&RedBlackNode`.
  pub fn find_mut(&mut self, key: &dyn DagNode) -> Option<&mut RedBlackNode> {
    let found: Option<&mut RedBlackNode> = self.rb_tree.get_mut(&(key.symbol().get_hash_value() as i32));
    if let Some(tree_node) = found {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      if key.compare(tree_node.dag_node.as_ref()) == Ordering::Equal {
        return Some(tree_node);
      }
    }

    return None;
  }

  ///	Return a cursor to the leftmost RedBlackNode that is a potential match for key, given the partial substitution
  /// for key's variables.
  // Todo: It looks like FreeTerm is the only subclass that overrides `Term::partialCompare()`. Nothing else seems to
  //       use the `partial` `Substitution` parameter at all, and as far as I can tell, `Term::partialCompare()`
  //       never returns `UNDECIDED`. Therefore, "partialCompare" is actually just "compare", and this method is find
  //       the g.l.b. of `key` in `self`.
  pub(crate) fn find_first_potential_match(&mut self, key: &dyn Term, _partial: &mut Substitution)
    -> Option<&RedBlackNode>
  {
    let numeric_key = key.symbol().get_hash_value() as i32;
    // Return maximum in map strictly less than `numeric_key`.
    self.rb_tree.range(..numeric_key).next_back().map(|(_, node)| node)
  }


  /// Return a mutable cursor pointing to the first node with multiplicity >= the given multiplicity.
  pub(crate) fn find_greater_equal_multiplicity(&mut self, multiplicity: u32) -> Option<&mut RedBlackNode> {
    self.rb_tree.values_mut().find(|&tree_node| *tree_node.multiplicity.borrow() >= multiplicity)
  }


  /// Moves all nodes of the tree into a vector, consuming self, and returns the vector.
  pub fn vectorize(mut self) -> Vec<DagPair> {
    Vec::from_iter(self.rb_tree.into_values().map(
      |tree_node|
      DagPair {
          dag_node: tree_node.dag_node,
          multiplicity: *tree_node.multiplicity.borrow()
        }
    ))
  }

  /// Deletes `multiplicity` copies of key.
  pub fn delete_multiplicity(&mut self, key: &dyn DagNode, multiplicity: u32) {
    let numeric_key = key.symbol().get_hash_value() as i32;

    if let Some(tree_node) = self.rb_tree.get_mut(&numeric_key){
      let new_multiplicity = *tree_node.multiplicity.borrow() - multiplicity;
      // Cannot delete more than exist.
      assert!(new_multiplicity >= 0);

      if new_multiplicity > 0 {
        // Leaving some remaining

        // Todo: There is no way to update max_multiplicity with this Red-Black tree implementation, because we don't
        //       have access to the left and right child nodes.
        // let mut max_multiplicity = new_multiplicity;

        *tree_node.multiplicity.borrow_mut() = new_multiplicity;

      } else {
        // We remove the node.
        self.rb_tree.remove(&numeric_key);
        // And adjust the size accordingly.
        self.size -= 1;
      }
    }

    // Todo: What if the key is not found?

  }
 

  /// Iterates over the nodes and their multiplicities.
  pub fn iter(&self) -> impl Iterator<Item=(RcDagNode, u32)> + '_ {
    // RedBlackTreeIterator::new(
    //   self.root.as_ref()
    // ).map(|node| (node.dag_node.as_ref(), node.multiplicity))
    self.rb_tree
        .iter()
        .map(|(_, node)|
            (
              node.dag_node.clone(),
              *node.multiplicity.borrow()
            )
        )
  }
}
