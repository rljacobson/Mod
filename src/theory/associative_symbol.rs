/*!

DEPRECATED.

This module contains structs shared by every associative theory.

 */


use std::{
  any::Any,
  fmt::Display
};

use crate::{
  theory::Symbol,
  core::{
    sort::{RcSort},
    pre_equation::sort_constraint_table::SortConstraintTable
  }
};

use super::symbol::SymbolMembers;


#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AssociativeSymbolStructure {
  Unstructured,
  // no guarantees
  LimitSort,
  // s_1 <= s & s_2 <= s ===> s_f(s_1, s_2) <= s
  PureSort,      // replaces ===> with <===>, taking sort constraints in to account
}

// Local convenience alias
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

pub struct AssociativeSymbol {
  pub sort_bounds: Vec<u32>,
  pub sort_structure: Vec<Structure>,
  pub uniform_sort: RcSort,

  // Symbol members
  sort: RcSort,
  sort_constraint_table: SortConstraintTable,

  pub symbol_members: SymbolMembers,
}


impl Symbol for AssociativeSymbol {
  fn symbol_members(&self) -> &SymbolMembers {
    &self.symbol_members
  }

  fn symbol_members_mut(&mut self) -> &mut SymbolMembers {
    &mut self.symbol_members
  }

  fn as_any(&self) -> &dyn Any {
    self
  }
}
