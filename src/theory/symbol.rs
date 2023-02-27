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


use crate::{
  sort_constraint::SortConstraintTable,
  theory::{
    RcDagNode,
    RcTerm
  }
};

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub struct Symbol {
//   pub order            : u32, // Unique integer for comparing symbols.
//   pub unique_sort_index: u32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
//   pub match_index      : u32, // For fast matching
//   pub arity            : u32,
//   pub memo_flag         : u32,
// }

pub trait Symbol {

  fn get_hash_value(&self) -> u32 {
    self.get_order()
  }

  fn get_order(&self) -> u32;

  fn get_sort_constraint_table(&self) -> &SortConstraintTable;

  fn sort_constraint_free(&self) -> bool {
    self.get_sort_constraint_table().is_empty()
  }

  fn compare(&self, other: &dyn Symbol) -> Ordering {
    // This is just std::Ord::cmp(self, other)
    self.cmp(other)
  }

}

impl PartialOrd for dyn Symbol {
  #[inline(always)]
  fn partial_cmp(&self, other: &dyn Symbol) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for dyn Symbol {
  #[inline(always)]
  fn cmp(&self, other: &dyn Symbol) -> Ordering {
    self.get_hash_value().cmp(&other.get_hash_value())
  }
}

impl Eq for dyn Symbol {}

impl PartialEq for dyn Symbol {
  #[inline(always)]
  fn eq(&self, other: &dyn Symbol) -> bool {
    self.get_hash_value() == other.get_hash_value()
  }
}


/*
Deriving Traits:
  BinarySymbol
*/


pub trait BinarySymbol: Symbol {
  fn get_identity(&self) -> RcTerm;
  fn get_identity_dag(&self) -> Option<RcDagNode>;
}
