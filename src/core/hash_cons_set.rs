/*!

This wraps an `IndexedHashSet<RcDagNode>` so that operations specific to canonicity are built in.

*/

use crate::abstractions::IndexedHashSet;
use crate::theory::RcDagNode;

#[derive(Default)]
pub struct HashConsSet {
  inner: IndexedHashSet<RcDagNode>
}


impl HashConsSet {

  pub fn insert(&mut self, node: RcDagNode) -> (RcDagNode, usize) {
    let (canonical, idx) = self.inner.insert_no_replace(node);
    canonical.borrow_mut().upgrade_sort_index(node.as_ref());
    (canonical, idx)
  }


}
