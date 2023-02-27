/*!

This module contains structs shared by every associative theory.

*/


use std::fmt::Display;
use crate::Sort;
use crate::sort_constraint::SortConstraintTable;
use crate::theory::Symbol;

pub enum AssociativeSymbolStructure {
  Unstructured, // no guarantees
  LimitSort,    // s_1 <= s & s_2 <= s ===> s_f(s_1, s_2) <= s
  PureSort      // replaces ===> with <===>, taking sort constraints in to account
}

type Structure = AssociativeSymbolStructure;

pub struct AssociativeSymbol<'s> {
  pub sort_bounds: Vec<u32>,
  pub sort_structure: Vec<Structure>,
  pub uniform_sort: &'s Sort,

  // Symbol members
  sort: Sort,
  sort_constraint_table: SortConstraintTable,

}

impl Display for Structure {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Structure::LimitSort => write!(f, "LIMIT_SORT"),
      Structure::PureSort => write!(f, "PURE_SORT"),
      Structure::Unstructured => write!(f, "UNSTRUCTURED")
    }
  }
}

impl Symbol for AssociativeSymbol<'_> {
    fn get_order(&self) -> u32 {
        self.sort
    }

    fn get_sort_constraint_table(&self) -> &SortConstraintTable {
        &self.sort_constraint_table
    }
}
