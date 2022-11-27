/*!


*/



use crate::theory::dag_node::DagNode;
use crate::theory::symbol::Symbol;

pub(crate) trait Term {
  fn symbol(&self) -> &Symbol;

  fn compare_dag_node(&self, other: &dyn DagNode) -> u32 {
    let value = self.symbol().compare(other.top_symbol());
    if value != 0 {
      self.compare_dag_arguments(other)
    } else {
      value
    }
  }

  fn compare_dag_arguments(&self, other: &DagNode) -> u32;
}

/*
Implementers:
  ACUTerm
*/
