/**

Variables have very minimal DAG nodes.

*/
use std::any::Any;
use std::cmp::Ordering;
use crate::abstractions::IString;
use crate::theory::dag_node::DagNodeMembers;
use crate::theory::{DagNode, DagNodeFlags, NodeList, RcSymbol};
use crate::theory::variable::symbol::VariableSymbol;


pub struct VariableDagNode {
  // Base DagNode Members
  pub members: DagNodeMembers,
  pub name: IString,
  pub index: u32,
}

impl VariableDagNode {
  pub fn new(symbol: RcSymbol, name: IString, index: u32) -> Self {
    let members = DagNodeMembers {
      top_symbol: symbol,
      args      : NodeList::new(),
      // sort      : None,
      flags     : DagNodeFlags::default(),
      sort_index: 0,
    };

    VariableDagNode {
      members,
      name,
      index
    }
  }
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
    if let Some(symbol) = self.members.top_symbol.as_any().downcast_ref::<VariableSymbol>() {
      let si = symbol.sort().borrow().sort_index;
      self.set_sort_index(si);
      return si;
    } else {
      unreachable!("Failed to downcast to VariableSymbol. This is a bug.");
    }
  }
}
