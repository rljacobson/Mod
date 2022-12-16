/*!



 */

use crate::cached_dag::CachedDag;
use crate::theory::{RcDagNode, Symbol};
use crate::theory::symbol::BinarySymbol;
use crate::theory::term::RcTerm;

pub struct ACUSymbol {
  pub order            : u32, // Unique integer for comparing symbols.
  pub unique_sort_index: i32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
  pub match_index      : u32, // For fast matching
  pub arity            : u32,
  pub memo_flag        : u32,

  identity : CachedDag, // Supposed to be a CachedDag
}


impl Symbol for ACUSymbol {
  fn get_order(&self) -> u32 {
    self.order
  }
}

impl BinarySymbol for ACUSymbol {
  fn get_identity(&self) -> Option<RcTerm> {
    self.identity.term.clone()
  }

  fn get_identity_dag(&self) -> Option<RcDagNode> {
    self.identity.dag_node.clone()
  }
}
