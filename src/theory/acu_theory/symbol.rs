/*!



 */

use std::rc::Rc;

use dyn_clone::{clone_trait_object, DynClone};

use crate::{
  sort::RcSort,
  cached_dag::CachedDag,
  theory::{
    BinarySymbol,
    DagNode,
    RcDagNode,
    RcTerm,
    Symbol,
  }
};

pub type RcACUSymbol = Rc<ACUSymbol>;

#[derive(Clone)]
pub struct ACUSymbol {
  pub hash_value       : u32, // Unique integer for comparing symbols, also called order
  pub unique_sort_index: i32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
  pub match_index      : u32, // For fast matching
  pub arity            : u32,
  pub memo_flag         : u32,

  identity : CachedDag, // Supposed to be a CachedDag
}


impl Symbol for ACUSymbol {
    fn get_hash_value(&self) -> u32 {
        self.hash_value
    }

    // fn compute_base_sort(&self, subject: &mut dyn DagNode) {
    //     // #[cfg(feature="DEBUG")]
    //     assert!(*self==*subject.symbol());
    //     let sort_index = subject.compute_base_sort();
    //     subject.set_sort_index(sort_index);
    // }

    fn get_sort_constraint_table(&self) -> &crate::sort_constraint::SortConstraintTable {
        todo!()
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
