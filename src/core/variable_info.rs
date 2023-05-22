/*!

Information about a variable that gets passed down through the compilation functions.

*/


use std::collections::HashSet;
use std::ops::Index;
use pratt::Channel::Debug;
use pratt::log;

use crate::abstractions::{NatSet, Graph};
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
  protected_variable_count: u32,
  fragment_number         : u32,
  construction_indices    : Vec<ConstructionIndex>,
  condition_variables     : NatSet,
  pub(crate) unbound_variables       : NatSet,
}

impl VariableInfo {

  pub fn new() -> Self {
    Self::default()
  }

  // region Accessors

  pub fn real_variable_count(&self) -> usize {
    self.variables.len()
  }

  pub fn protected_variable_count(&self) -> i32 {
    self.protected_variable_count as i32
  }

  pub(crate) fn index2variable(&self, index: usize) -> RcTerm {
    self.variables[index].clone()
  }

  pub(crate) fn variable_to_index(&mut self, variable: RcTerm) -> i32 {
    // assert!(variable != &VariableTerm::default(), "null term");
    assert!(self.variables.len() == self.protected_variable_count as usize, "can't add new real variables at this stage");

    let idx = self.variables.iter().position(|v| v.borrow().compare(&*variable.borrow()).is_eq());
    match idx {
      Some(i) => i as i32,
      None => {
        self.variables.push(variable.clone());
        self.protected_variable_count += 1;
        (self.variables.len() - 1) as i32
      }
    }
  }

  /// The phrase "remap index" is a noun. This method is a const getter and does not actually compute the remapping. Use
  /// `compute_index_remapping` to compute the remap index.
  pub fn remap_index(&self, original: usize) -> u32 {
    if original >= MAX_NR_PROTECTED_VARIABLES {
      self.construction_indices[(original - MAX_NR_PROTECTED_VARIABLES) as usize].new_index
    } else {
      original as u32
    }
  }

  pub fn make_protected_variable(&mut self) -> u32 {
    self.protected_variable_count += 1;
    self.protected_variable_count - 1
  }

  pub fn end_of_fragment(&mut self) {
    self.fragment_number += 1;
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
    self.condition_variables.union_in_place(vars);
  }

  pub fn add_unbound_variables(&mut self, vars: &NatSet) {
    self.unbound_variables.union_in_place(vars);
  }

  // endregion Accessors

  pub(crate) fn compute_index_remapping(&mut self) -> i32 {
    let construction_indices_count = self.construction_indices.len();

    // All construction indices that need to be protected between different fragments
    // get remapped to a new protected variable.
    { // scope of new_protected_variable_count
      let mut new_protected_variable_count = self.protected_variable_count;
      for mut idx in self.construction_indices.iter_mut() {
        if idx.assigned_fragment != idx.last_use_fragment {
          idx.new_index = new_protected_variable_count;
          new_protected_variable_count += 1;
        }
      }
      self.protected_variable_count = new_protected_variable_count;
    }

    // We now build a graph of conflicts between remaining construction indices.
    #[cfg(debug_assertions)]
    if !(construction_indices_count < 100){
      log(Debug, 3, format!("nrConstructionIndices = {}", construction_indices_count).as_str())
    }
    let mut conflicts: Graph = Graph::new(construction_indices_count);
    let mut conflict_candidates = Vec::new();
    let mut next_conflict_candidates = Vec::new();
    for i in 0..construction_indices_count {
      if self.construction_indices[i].assigned_fragment == self.construction_indices[i].last_use_fragment {
        // A remaining construction index i conflicts with any earlier
        // remaining construction index j whose last use is after the
        // allocation of construction index i. To speed things up
        // when the number of construction indices is huge, we keep track
        // of a smaller pool of candidates.
        next_conflict_candidates.clear();
        for &c in &conflict_candidates {
          let construction_index: ConstructionIndex = self.construction_indices[c];
          if construction_index.last_use_time as usize > i {
            conflicts.insert_edge(i, c);
            next_conflict_candidates.push(c);
          }
        }
        next_conflict_candidates.push(i);
        std::mem::swap(&mut conflict_candidates, &mut next_conflict_candidates);
      }
    }

    // We now use graph coloring to remap the remaining construction indices.
    let mut coloring = vec![0; construction_indices_count];
    let color_count = conflicts.color(&mut coloring);
    for i in 0..construction_indices_count {
      if self.construction_indices[i].assigned_fragment == self.construction_indices[i].last_use_fragment {
        self.construction_indices[i].new_index = self.protected_variable_count + coloring[i] as u32;
      }
    }

    // Finally, we need to return the minimum size of substitution needed.
    self.protected_variable_count as i32 + color_count
    /*
    DebugAdvisory("nrProtectedVariables = " << nrProtectedVariables <<
                  "\tnrColors = " << nrColors);
    */
  }

}



impl Index<usize> for VariableInfo {
  type Output = RcTerm;

  fn index(&self, index: usize) -> &Self::Output {
    &self.variables[index]
  }
}
