/*!

DEPRECATED.

This module contains structs shared by every associative theory.

*/


use std::fmt::Display;
use crate::Sort;
use crate::sort_constraint::SortConstraintTable;
use crate::theory::Symbol;

#[derive(Clone)]
pub struct AssociativeSymbol<'s> {
  pub sort_bounds: Vec<u32>,
  pub sort_structure: Vec<Structure>,
  pub uniform_sort: &'s Sort,

  // Symbol members
  sort: Sort,
  sort_constraint_table: SortConstraintTable,

  // Unique integer for comparing symbols, also called the order elsewhere in the code.
  pub hash_value            : u32,

//   pub unique_sort_index: u32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
//   pub match_index      : u32, // For fast matching
//   pub arity            : u32,
//   pub memo_flag         : u32,

}



impl Symbol for AssociativeSymbol<'_> {

    fn get_sort_constraint_table(&self) -> &SortConstraintTable {
        &self.sort_constraint_table
    }

    fn get_hash_value(&self) -> u32 {
        self.hash_value
    }

    fn compute_base_sort(&self, subject: &mut dyn super::DagNode) {
        todo!()
    }

}
