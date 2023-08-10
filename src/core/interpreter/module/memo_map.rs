/*!

Memoization map for all symbols in a module.


*/

use crate::abstractions::DagNodeHashSet;
use crate::theory::RcDagNode;

pub struct MemoMap {
  dags: DagNodeHashSet,
  /// Maps from-indices to to-indices
  to_indices: Vec<Option<i32>>,
}

impl MemoMap {
  pub fn get_from_index(&mut self, from_dag: RcDagNode) -> i32 {
    // We assume that a from_dag is unreduced, and therefore we never use
    // the original in the hash cons table in case it is reduced in place.
    let from_index = self.dags.insert_copy(from_dag);
    let from_dags_count = self.to_indices.len();
    if from_index >= from_dags_count {
      self.to_indices.resize(from_index + 1, None);
    }

    return from_index;
  }

  pub fn get_to_dag(&self, from_index: i32) -> Option<RcDagNode> {
    match self.to_indices[from_index as usize] {
      Some(to_index) => Some(self.dags.get_canonical(to_index)),
      None => None,
    }
  }

  pub fn assign_to_dag(&mut self, from_index: i32, to_dag: RcDagNode) {
    self.to_indices[from_index as usize] = Some(self.dags.insert(to_dag));
  }
}
