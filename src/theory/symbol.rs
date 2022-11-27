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
use crate::theory::dag_node::DagNode;

pub struct Symbol {
  pub order            : u32, // Unique integer for comparing symbols.
  pub unique_sort_index: u32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
  pub match_index      : u32, // For fast matching
  pub arity            : u32,
  pub memo_flag        : u32,
}

impl Symbol {

  fn get_hash_value(&self) -> u32 {
    self.order
  }

  pub(crate) fn compare(&self, other: &Self) -> u32 {
    self.get_hash_value() - other.get_hash_value()
  }

}

