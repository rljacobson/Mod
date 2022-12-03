/*!



 */
use std::borrow::BorrowMut;
use std::cell::Cell;
use intrusive_collections::{RBTreeLink, RBTree, KeyAdapter, intrusive_adapter, Adapter, Bound};
use intrusive_collections::rbtree::Color::Red;
use intrusive_collections::rbtree::{Cursor, CursorMut};

use crate::{Substitution, theory::{
  DagNode,
  Term,
  DagPair
}};
use crate::theory::dag_node::BcDagNode;

use super::RedBlackNode;

intrusive_adapter!(RBTreeAdapter = Cell<RedBlackNode>: RedBlackNode { link: RBTreeLink });
impl<'a> KeyAdapter<'a> for RBTreeAdapter {
  type Key = u32;
  fn get_key(&self, x: &'a RedBlackNode) -> u32 { x.dag_node.symbol().get_hash_value() }
}



#[derive(Clone)]
pub struct RedBlackTree {
  // root: Box<RedBlackNode>,
  rb_tree: RBTree<RBTreeAdapter>,
  pub(crate) size: usize, // Todo: Construction methods need to update this.
}

impl RedBlackTree {

  pub fn new(root: BcDagNode, multiplicity: u32) -> Self {
    let mut tree = RedBlackTree {
      rb_tree: RBTree::new(RBTreeAdapter::new()),
      size: 0,
    };
    let root = Cell::new(
      RedBlackNode::new(root, multiplicity)
    );

    tree.rb_tree.insert(root);
    tree
  }

  // todo: Does this need to return a path? Can it return the DagNode? The cursor?
  /// If found, returns a cursor to the node for the key.
  pub fn find_mut(&mut self, key: &dyn Term) -> Option<CursorMut<RBTreeAdapter>> {
    let mut cursor: CursorMut<RBTreeAdapter> = self.rb_tree.find_mut(&key.symbol().get_hash_value());
    if let Some(found) = cursor.get() {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      let r =  key.compare_dag_node(Cell::<RedBlackNode>::get((found)).dag_node.as_ref());
      if r ==  0 {
        return Some(cursor);
      }
    }

    return None;
  }

  /// Same as above, but returns a `CursorMut` instead of a `Cursor`.
  pub fn find(&self, key: &dyn Term) -> Option<Cursor<RBTreeAdapter>> {
    let mut cursor: Cursor<RBTreeAdapter> = self.rb_tree.find(&key.symbol().get_hash_value());
    if let Some(found) = cursor.get() {
      // The result, if exists, just has same top symbol. Now compare arguments as well.
      let r =  key.compare_dag_node(Cell::<RedBlackNode>::get((found)).dag_node.as_ref());
      if r ==  0 {
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
  pub(crate) fn find_first_potential_match(&mut self, key: &dyn Term, partial: &mut Substitution)
    -> Option<CursorMut<RBTreeAdapter>>
  {
    let mut cursor = self.rb_tree.lower_bound_mut(Bound::Included(&key.symbol().get_hash_value()));
    if cursor.is_null(){
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
      let mut node = Cell::<RedBlackNode>::get(node_ptr);
      vector.push(
        DagPair {
          dag_node: node.dag_node,
          multiplicity: node.multiplicity
        }
      );
    }

    vector
  }

  /// Deletes `multiplicity` copies of key.
  pub fn delete_multiplicity(&mut self, key: &dyn DagNode, multiplicity: u32) {
    let mut cursor = self.rb_tree.find_mut(&key.symbol().get_hash_value());
    if !cursor.is_null(){
      self.delete_multiplicity_at_cursor(&mut cursor, multiplicity)
    }
  }

  /// Same as `delete_multiplicity(..)` but takes a cursor pointing to the dagnode to delete. This avoids a search of
  /// the tree for the DagNode. Whether or not the original node
  pub fn delete_multiplicity_at_cursor(&mut self, cursor: &mut CursorMut<RBTreeAdapter>, multiplicity: u32) {
    // let mut cursor = self.rb_tree.find_mut(&key.symbol().get_hash_value());
    if let Some(victim_ptr) = cursor.get() {
      let victim: &mut RedBlackNode = Cell::<RedBlackNode>::get_mut(*victim_ptr);
      let new_mult = victim.multiplicity - multiplicity;
      if new_mult > 0 {
        victim.multiplicity = new_mult;
        // cursor.insert(victim);
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
  pub fn iter(&self) -> impl Iterator<Item=(&dyn DagNode, u32)> {
    // RedBlackTreeIterator::new(
    //   self.root.as_ref()
    // ).map(|node| (node.dag_node.as_ref(), node.multiplicity))
    self.rb_tree
        .iter()
        .map(|node|
            (
              Cell::<RedBlackNode>::get(node).dag_node.as_ref(),
              Cell::<RedBlackNode>::get(node).multiplicity
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
