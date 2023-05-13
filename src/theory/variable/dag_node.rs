/**

Variables have very minimal DAG nodes.

*/
use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use crate::{theory::{
  DagNode,
  dag_node_flags,
  NodeList,
  RcSymbol,
  RcTerm,
  DagNodeFlags,
  RcDagNode,
  variable::{
    VariableSymbol,
    VariableTerm
  },
  dag_node::DagNodeMembers,
}, abstractions::{IString, RcCell}, rc_cell};
use crate::core::RedexPosition;


pub struct VariableDagNode {
  // Base DagNode Members
  pub members: DagNodeMembers,
  pub name: IString,
  pub index: i32,
}

impl VariableDagNode {
  pub fn new(symbol: RcSymbol, name: IString, index: i32) -> Self {
    let members = DagNodeMembers {
      top_symbol: symbol,
      args      : NodeList::new(),
      // sort      : None,
      flags     : DagNodeFlags::default(),
      sort_index: -1,
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

  fn termify(&self) -> RcTerm {
    rc_cell!(VariableTerm::new(self.name.clone(), self.symbol()))
  }


  fn shallow_copy(&self) -> RcDagNode {
    // There are no args, so just make a new one.
    let mut fdg = VariableDagNode::new(self.symbol(), self.name.clone(), self.index);
    fdg.set_flags(self.flags() & DagNodeFlags::RewritingFlags);

    rc_cell!(fdg)
  }

  fn copy_with_replacements(&self, _stack: &[RedexPosition], _first_idx: usize, _last_idx: usize) -> RcDagNode {
    unreachable!("This execution path should be unreachable. This is a bug.")
  }

  fn copy_with_replacement(&self, _replacement: RcDagNode, _arg_index: usize) -> RcDagNode {
    unreachable!("This execution path should be unreachable. This is a bug.")
  }
}
