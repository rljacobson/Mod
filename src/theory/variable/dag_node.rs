/**

Variables have very minimal DAG nodes.

*/
use std::any::Any;
use std::cmp::Ordering;
use crate::abstractions::IString;
use crate::theory::dag_node::DagNodeMembers;
use crate::theory::DagNode;


pub struct VariableDagNode {
  // Base DagNode Members
  pub members: DagNodeMembers,
  pub name: IString,
  pub index: u32,
}

impl DagNode for VariableDagNode {
  fn dag_node_members(&self) -> &DagNodeMembers {
    &self.members
  }

  fn dag_node_members_mut(&mut self) -> &mut DagNodeMembers {
    &mut self.members
  }

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_ptr(&self) -> *const dyn DagNode {
    self as *const dyn DagNode
  }

  fn compare_arguments(&self, other: &dyn DagNode) -> Ordering {
    if let Some(other) = other.as_any().downcast_ref::<VariableDagNode>(){
      self.name.cmp(&other.name)
    } else {
      Ordering::Less
    }
  }

  fn compute_base_sort(&mut self) -> i32 {
    let si = self.members.sort.borrow().sort_index;
    self.set_sort_index(si);
    si
  }
}
