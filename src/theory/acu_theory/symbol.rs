/*!



 */

use std::rc::Rc;

use dyn_clone::{clone_trait_object, DynClone};

use crate::{
  sort::RcSort,
  sort_constraint::SortConstraintTable,
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AssociativeSymbolStructure {
  Unstructured, // no guarantees
  LimitSort,    // s_1 <= s & s_2 <= s ===> s_f(s_1, s_2) <= s
  PureSort      // replaces ===> with <===>, taking sort constraints in to account
}

type Structure = AssociativeSymbolStructure;


impl Display for Structure {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Structure::LimitSort => write!(f, "LIMIT_SORT"),
      Structure::PureSort => write!(f, "PURE_SORT"),
      Structure::Unstructured => write!(f, "UNSTRUCTURED")
    }
  }
}


#[derive(Clone)]
pub struct ACUSymbol {
  identity : CachedDag, // Supposed to be a CachedDag

  // AssociativeSymbol members
  pub sort_bounds       : Vec<u32>,
  pub sort_structure    : Vec<Structure>,
  pub uniform_sort      : &'s Sort,
  sort                  : Sort,
  sort_constraint_table : SortConstraintTable,
  pub hash_value        : u32, // Unique integer for comparing symbols, also called order
  pub unique_sort_index : i32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
  pub match_index       : u32, // For fast matching
  pub arity             : u32,
  pub memo_flag         : u32,
}


impl Symbol for ACUSymbol {
    fn get_hash_value(&self) -> u32 {
        self.hash_value
    }

    // TODO: Weird code smell. Doesn't even use `self`.
    // fn compute_base_sort(&self, subject: &mut dyn DagNode) {
    //     // #[cfg(feature="DEBUG")]
    //     assert!(*self==*subject.symbol());
    //     let sort_index = subject.compute_base_sort();
    //     subject.set_sort_index(sort_index);
    // }

    fn get_sort_constraint_table(&self) -> &SortConstraintTable {
        &self.sort_constraint_table
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
