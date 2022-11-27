/*!

Concrete types for the ACU theory implementing the DagNode trait.

*/

use std::any::Any;
use std::borrow::BorrowMut;
use std::slice::Iter;

use crate::theory::dag_node::{DagNode, DagPair};
use crate::theory::symbol::Symbol;
use crate::theory::term::Term;
use super::red_black_tree::{RedBlackTree, RedBlackNode};

enum ACUArguments {
  List(Vec<DagPair>),
  Tree(RedBlackTree)
}

impl ACUArguments {
  pub fn size(&self) -> u32 {
    match self {
      ACUArguments::List(v) => { v.len() as u32 }
      ACUArguments::Tree(t) => { t.size() }
    }
  }

  /// Alias for size()
  pub fn len(&self) -> u32 {
    self.size()
  }
}

impl ACUArguments {
  /// Searches for the given term and returns its multiplicity if found.
  pub fn search_for_term(&self, term: &Term) -> Option<u32> {
    match self {

      ACUArguments::List(v) => {
        v.binary_search_by((|(t, m)| t.compare(term))).map(|idx| v[idx].1).ok()
      },

      ACUArguments::Tree(t) => {
        t.node_for_key(term)
      }
    }
  }

  pub fn iter(&self) -> Iter<DagPair> {
    match self {
      ACUArguments::List(v) => {
        v.iter()
      },
      ACUArguments::Tree(t) => {
        t.iter()
      }
    }
  }

}


pub(crate) struct ACUDagNode {
  top_symbol: Box<Symbol>,
  args: ACUArguments
}

impl ACUDagNode {
  //	Returns index of argument equal key, or a -ve value pos otherwise.
  //	In the latter case ~pos is the index of the smallest element greater
  //	than key, and can be argArray.length() if key is greater than all elements
  //	in the array.
  pub fn binary_search_by_term(&self, key: &dyn Term) -> Option<Vec<&Box<RedBlackNode>>> {
    let mut upper = self.args.len();
    let mut lower: usize = 0;

    loop {
      let sum = upper + lower;
      let probe = sum/2;

      let r =  key.compare_dag_node(&self.args[probe]);
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

impl DagNode for ACUDagNode {
  fn top_symbol(&self) -> &Symbol {
    self.top_symbol.as_ref()
  }

  fn top_symbol_mut(&mut self) -> &mut Symbol {
    self.top_symbol.borrow_mut()
  }

  fn args(&self) -> Iter<DagPair> {
    self.iter()
  }

  fn as_any(&self) -> &dyn Any{
    self
  }
}


