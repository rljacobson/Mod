/*!



 */
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::rc::Rc;

use intrusive_collections::{
  RBTreeLink,
  RBTree,
  KeyAdapter,
  intrusive_adapter,
  Adapter,
  Bound,
  rbtree::{Cursor, CursorMut}
};
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


intrusive_adapter!(pub RBTreeAdapter = Rc<RedBlackNode>: RedBlackNode { link: RBTreeLink });
impl<'a> KeyAdapter<'a> for RBTreeAdapter {
  type Key = u32;
  fn get_key(&self, x: &'a RedBlackNode) -> u32 { x.dag_node.symbol().get_hash_value() }
}


pub struct RedBlackTree {
  // root: &'a Cell<RedBlackNode>,
  rb_tree: RBTree<RBTreeAdapter>,
  pub(crate) size: usize, // Todo: Construction methods need to update this.
}

impl RedBlackTree {

  pub fn new(root: RcDagNode, multiplicity: u32) -> Self {
    let root = Rc::new(
      RedBlackNode::new(root, multiplicity)
    );

    let mut tree = RedBlackTree {
      // root: root.borrow(),
      rb_tree: RBTree::new(RBTreeAdapter::new()),
      size   : 0,
    };

    tree.rb_tree.insert(root);
    tree
  }

  pub fn clear(&mut self) {
    self.rb_tree.clear();
    self.size = 0;
  }

  pub fn insert(&mut self, dag_node: RcDagNode) {
    let node = Rc::new(RedBlackNode::new(dag_node, 1));
    self.rb_tree.insert(node);
  }

  /// Gets the multiplicity of the first node in the tree. If size==1, that would be the only multiplicity in the tree.
  pub fn get_sole_multiplicity(&self) -> u32 {
    self.rb_tree.front().get().unwrap().multiplicity
  }

  /// Gets the first node in the tree. If size==1, that would be the only node in the tree.
  pub fn get_sole_dag_node(&self) -> RcDagNode {
    self.rb_tree.front().get().unwrap().dag_node.clone()
  }

  pub fn is_reduced(&self) -> bool {
    // Todo: Imnplement `is_reduced()` for RedBlackTree. In Maude it is a property of a DagNode, and a tree is reduced if its
    //       root is.
    true
  }

  pub fn max_multiplicity(&self) -> u32 {
    self.rb_tree
        .iter()
        .map(|node| node.multiplicity)
        .max()
        .unwrap_or(0)
  }
  
  // Todo: Why not have `find*` methods take a `&Symbol` instead of a `Term` or `DagNode`? Then we'd only need one set.

  /// If found, returns a cursor to the node for the key.
  pub fn find_term(&self, key: &dyn Term) -> Option<Cursor<RBTreeAdapter>> {
    let mut cursor: Cursor<RBTreeAdapter> = self.rb_tree.find(&key.symbol().get_hash_value());
    if let Some(found) = cursor.get() {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      let r =  key.compare_dag_arguments(found.dag_node.as_ref());
      if r == Ordering::Equal {
        return Some(cursor);
      }
    }

    return None;
  }

  /// If found, returns a cursor to the node for the key.
  pub fn find(&self, key: &dyn DagNode) -> Option<Cursor<RBTreeAdapter>> {
    let mut cursor: Cursor<RBTreeAdapter> = self.rb_tree.find(&key.symbol().get_hash_value());
    if let Some(found) = cursor.get() {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      let r =  key.compare(found.dag_node.as_ref());
      if r ==  Ordering::Equal {
        return Some(cursor);
      }
    }

    return None;
  }

  /// Same as above, but returns a `CursorMut` instead of a `Cursor`.
  pub fn find_term_mut(&mut self, key: &dyn Term) -> Option<CursorMut<RBTreeAdapter>> {
    let mut cursor: CursorMut<RBTreeAdapter> = self.rb_tree.find_mut(&key.symbol().get_hash_value());
    if let Some(found) = cursor.get() {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      let r =  key.compare_dag_node(found.dag_node.as_ref());
      if r == Ordering::Equal {
        return Some(cursor);
      }
    }

    return None;
  }

  /// Same as above, but returns a `CursorMut` instead of a `Cursor`.
  pub fn find_mut(&mut self, key: &dyn DagNode) -> Option<CursorMut<RBTreeAdapter>> {
    let mut cursor: CursorMut<RBTreeAdapter> = self.rb_tree.find_mut(&key.symbol().get_hash_value());
    if let Some(found) = cursor.get() {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      let r =  key.compare(found.dag_node.as_ref());
      if r == Ordering::Equal {
        return Some(cursor);
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
    -> Option<CursorMut<RBTreeAdapter>>
  {
    let mut cursor = self.rb_tree.upper_bound_mut(Bound::Included(&key.symbol().get_hash_value()));
    if cursor.is_null(){
      None
    } else {
      Some(cursor)
    }
  }


  /// Return a mutable cursor pointing to the first node with multiplicity >= the given multiplicity.
  pub(crate) fn find_greater_equal_multiplicity(&mut self, multiplicity: u32) -> Option<CursorMut<RBTreeAdapter>> {
    let mut cursor = self.rb_tree.lower_bound_mut(Bound::Included(&multiplicity));

    if cursor.is_null() {
      None
    } else {
      Some(cursor)
    }
  }


  /// Moves all nodes of the tree into a vector, consuming self, and returns the vector.
  pub fn vectorize(mut self) -> Vec<DagPair> {
    let mut vector = Vec::with_capacity(self.size);
    let mut cursor = self.rb_tree.front_mut();

    // This isn't very efficient, but deletion has amortized complexity O(1), so it's probably okay.
    while let Some(mut node_ptr) = cursor.remove() {
      // let mut node = Cell::<RedBlackNode>::get(node_ptr);
      vector.push(
        DagPair {
          dag_node: node_ptr.dag_node,
          multiplicity: node_ptr.multiplicity
        }
      );
    }

    vector
  }

  /// Deletes `multiplicity` copies of key.
  pub fn delete_multiplicity(&mut self, key: &dyn DagNode, multiplicity: u32) {
    let mut cursor = self.rb_tree.find_mut(&key.symbol().get_hash_value());
    if !cursor.is_null(){
      self.delete_multiplicity_at_cursor(cursor, multiplicity);
    }
  }

  /// Same as `delete_multiplicity(..)` but takes a cursor pointing to the dagnode to delete. This avoids a search of
  /// the tree for the DagNode. Whether or not the original node
  pub fn delete_multiplicity_at_cursor(&mut self, mut cursor: CursorMut<RBTreeAdapter>, multiplicity: u32) {
    // let mut cursor = self.rb_tree.find_mut(&key.symbol().get_hash_value());
    if let Some(victim_ref) = cursor.get() {
      // CursorMut::get(&self)

      let new_multiplicity = victim_ref.multiplicity - multiplicity;
      // Cannot delete more than exist.
      assert!(new_multiplicity >= 0);

      if new_multiplicity > 0 {
        // Leaving some remaining

        // Todo: There is no way to update max_multiplicity with this Red-Black tree implementation, because we don't
        //       have access to the left and right child nodes.
        // let mut max_multiplicity = new_multiplicity;

        let replacement = RedBlackNode::new(victim_ref.dag_node.clone(), new_multiplicity);
        cursor.replace_with(Rc::new(replacement));

      } else {
        // We remove the node.
        cursor.remove();
        // And adjust the size accordingly.
        self.size -= 1;
      }

    }
  }

  /*
  // / Deletes `multiplicity` copies of key. Returns the amount by which self.size changed.
  pub fn cons_delete(&mut self, mut key: &dyn DagNode, multiplicity: u32)  {
    let mut cursor = self.rb_tree.find_mut(&key.symbol().get_hash_value());
    if let Some(mut victim) = cursor.remove() {
      let new_mult = victim.multiplicity - multiplicity;
      if new_mult > 0 {
        victim.borrow_mut().multiplicity = new_mult;
        cursor.insert(victim);
      }
      // We removed the node.
      self.size -= 1;
    }
  }
*/

  /// Iterates over the nodes and their multiplicities.
  pub fn iter(&self) -> impl Iterator<Item=(RcDagNode, u32)> + '_ {
    // RedBlackTreeIterator::new(
    //   self.root.as_ref()
    // ).map(|node| (node.dag_node.as_ref(), node.multiplicity))
    self.rb_tree
        .iter()
        .map(|node|
            (
              node.dag_node.clone(),
              node.multiplicity
            )
        )
  }
}

/*
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
*/
