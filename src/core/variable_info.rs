/*!

Information about a variable that gets passed down through the compilation functions.

*/


use std::collections::HashSet;
use std::ops::Index;

use crate::abstractions::NatSet;
use crate::theory::{RcTerm};


/// This is the boundary between real and virtual variables. An `index` represents a real variable
/// iff `index < MAX_NR_PROTECTED_VARIABLES`.
const MAX_NR_PROTECTED_VARIABLES: usize = 10_000_000;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
struct ConstructionIndex {
  last_use_time    : u32,
  assigned_fragment: i16,
  last_use_fragment: i16,
  new_index        : u32,
}

#[derive(Default)]
pub struct VariableInfo {
  variables               : Vec<RcTerm>,
  protected_variable_count: usize,
  fragment_number         : u32,
  construction_indices    : Vec<ConstructionIndex>,
  condition_variables     : NatSet,
  unbound_variables       : NatSet,
}

impl VariableInfo {

  pub fn new() -> Self {
    Self::default()
  }

  pub fn get_nr_real_variables(&self) -> usize {
    self.variables.len()
  }

  pub fn get_nr_protected_variables(&self) -> usize {
    self.protected_variable_count
  }

  fn index2variable(&self, index: usize) -> RcTerm {
    self.variables[index].clone()
  }

  pub fn make_protected_variable(&mut self) -> usize {
    self.protected_variable_count += 1;
    self.protected_variable_count - 1
  }

  pub fn end_of_fragment(&mut self) {
    self.fragment_number += 1;
  }

  pub fn remap_index(&self, original: usize) -> u32 {
    if original >= MAX_NR_PROTECTED_VARIABLES {
      self.construction_indices[(original - MAX_NR_PROTECTED_VARIABLES) as usize].new_index
    } else {
      original as u32
    }
  }

  pub fn use_index(&mut self, index: usize) {
    if index >= MAX_NR_PROTECTED_VARIABLES {
      let index = (index - MAX_NR_PROTECTED_VARIABLES) as usize;

      self.construction_indices[index].last_use_time = self.construction_indices.len() as u32;
      self.construction_indices[index].last_use_fragment = self.fragment_number as i16;
    }
  }

  pub fn get_condition_variables(&self) -> &NatSet {
    &self.condition_variables
  }

  pub fn get_unbound_variables(&self) -> &NatSet {
    &self.unbound_variables
  }

  pub fn add_condition_variables(&mut self, vars: &NatSet) {
    self.condition_variables.union_with(vars);
  }

  pub fn add_unbound_variables(&mut self, vars: &NatSet) {
    self.unbound_variables.union_with(vars);
  }

}



impl Index<usize> for VariableInfo {
  type Output = RcTerm;

  fn index(&self, index: usize) -> &Self::Output {
    &self.variables[index]
  }
}
