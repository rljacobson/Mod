/*!

The Symbol trait and its concrete implementations BinarySymbol and AssociativeSymbol.

A Symbol implements the traits:
    RuleTable,
    NamedEntity,
    LineNumber,
    SortTable,
    SortConstraintTable,
    EquationTable,
    Strategy,
    MemoTable

*/

use std::cmp::{Ordering, PartialOrd, Ord, Eq, PartialEq};

use dyn_clone::{clone_trait_object, DynClone};

use crate::{
  sort_constraint::SortConstraintTable,
  theory::{
    RcDagNode,
    RcTerm
  }, Sort, sort::RcSort
};

use super::DagNode;

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub struct Symbol {
//   pub order            : u32, // Unique integer for comparing symbols.
//   pub unique_sort_index: u32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
//   pub match_index      : u32, // For fast matching
//   pub arity            : u32,
//   pub memo_flag         : u32,
// }

pub trait Symbol: DynClone {

  fn get_hash_value(&self) -> u32;

  // fn get_order(&self) -> u32;

  // fn compute_base_sort(&self, subject: &mut dyn DagNode);


  fn get_sort_constraint_table(&self) -> &SortConstraintTable;

  fn sort_constraint_free(&self) -> bool {
    self.get_sort_constraint_table().is_empty()
  }

  fn compare(&self, other: &dyn Symbol) -> Ordering {
    // This is just std::Ord::cmp(self, other)
    // Ord::cmp(&self, other)
    self.get_hash_value().cmp(&other.get_hash_value())
  }

}

impl PartialOrd for dyn Symbol {

  fn partial_cmp(&self, other: &dyn Symbol) -> Option<Ordering> {
    let result = self.get_hash_value().cmp(&other.get_hash_value());
    Some(result)
  }
}

impl Ord for dyn Symbol {
  fn cmp(&self, other: &dyn Symbol) -> Ordering {
    self.get_hash_value().cmp(&other.get_hash_value())
  }
}

impl Eq for dyn Symbol {}

impl PartialEq for dyn Symbol {
  fn eq(&self, other: &dyn Symbol) -> bool {
    self.get_hash_value() == other.get_hash_value()
  }
}


clone_trait_object!(Symbol);

/*
Deriving Traits:
  BinarySymbol
*/


pub trait BinarySymbol: Symbol {
  fn get_identity(&self) -> Option<RcTerm>;
  fn get_identity_dag(&self) -> Option<RcDagNode>;
}
