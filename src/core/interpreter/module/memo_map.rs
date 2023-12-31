/*!

Memoization map for all symbols in a module.

It's not clear how this is supposed to work.

hash_cons_set->pointer2Index(*dagnode, dagnode->hash_value) gives __index__
pounter_set->insert(*dagnode, dagnode->hash_value) gives __index__ ?

*/

use std::rc::Rc;

use crate::abstractions::DagNodeHashSet;
use crate::theory::RcDagNode;

pub type RcMemoMap = Rc<MemoMap>;
pub type BxMemoMap = Box<MemoMap>;

#[derive(Default)]
pub struct MemoMap {
  dags: DagNodeHashSet,
  /// Maps from-indices to to-indices
  to_indices: Vec<Option<i32>>,
}

impl MemoMap {
  pub fn get_from_index(&mut self, from_dag: RcDagNode) -> i32 {
    // We assume that a from_dag is unreduced, and therefore we never use
    // the original in the hash cons table in case it is reduced in place.
    let (_, from_index) = self.dags.insert_copy(from_dag);
    let from_dags_count = self.to_indices.len();
    // ToDO: We're comparing a hash value to a length. This is nonsense.
    assert!(false);
    if from_index >= from_dags_count as u64 {
      self.to_indices.resize(from_index as usize + 1, None);
    }

    return from_index as i32;
  }

  // pub fn get_to_dag(&self, from_index: i32) -> Option<RcDagNode> {
  //   match self.to_indices[from_index as usize] {
  //     Some(to_index) => Some(self.dags.get_canonical(to_index)),
  //     None => None,
  //   }
  // }

  pub fn assign_to_dag(&mut self, from_index: i32, to_dag: RcDagNode) {
    let (_, hash) = self.dags.insert(to_dag);
    self.to_indices[from_index as usize] = Some(hash as i32);
  }
}
