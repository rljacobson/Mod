/**

Variables have very minimal DAG nodes.

*/
use std::{any::Any, cell::RefCell, cmp::Ordering, ops::Deref, rc::Rc};

use crate::{
  abstractions::{IString, RcCell},
  core::{hash_cons_set::HashConsSet, RedexPosition},
  rc_cell,
  theory::{
    dag_node_flags,
    variable::{VariableSymbol, VariableTerm},
    DagNode,
    DagNodeFlag,
    DagNodeFlags,
    DagNodeMembers,
    NodeList,
    RcDagNode,
    RcSymbol,
    RcTerm,
  },
};


pub struct VariableDagNode {
  // Base DagNode Members
  pub members: DagNodeMembers,
  pub name:    IString,
  pub index:   i32,
}

impl VariableDagNode {
  pub fn new(symbol: RcSymbol, name: IString, index: i32) -> Self {
    let members = DagNodeMembers {
      top_symbol: symbol,
      args:       NodeList::new(),
      flags:      DagNodeFlags::default(),
      sort_index: -1,
      copied_rc:  None,
      hash:       0,
    };

    VariableDagNode { members, name, index }
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
    if let Some(other) = other.as_any().downcast_ref::<VariableDagNode>() {
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

  fn copy_eager_upto_reduced_aux(&mut self) -> RcDagNode {
    rc_cell!(VariableDagNode::new(self.symbol(), self.name, self.index))
  }

  fn copy_all_aux(&mut self) -> RcDagNode {
    rc_cell!(VariableDagNode::new(self.symbol(), self.name, self.index))
  }

  fn overwrite_with_clone(&mut self, mut old: RcDagNode) {
    if let Some(old_dag_node) = old.borrow_mut().as_any_mut().downcast_mut::<VariableDagNode>() {
      let mut fdg = VariableDagNode::new(self.symbol(), self.name.clone(), self.index);
      fdg.set_flags(
        self.flags()
          | DagNodeFlag::Reduced
          | DagNodeFlag::Unrewritable
          | DagNodeFlag::Unstackable
          | DagNodeFlag::Ground,
      );

      let _ = std::mem::replace(old_dag_node, fdg);
    } else {
      unreachable!("This execution path should be unreachable. This is a bug.")
    }
  }

  fn make_canonical(&self, node: RcDagNode, _: &mut HashConsSet) -> RcDagNode {
    node
  }
}
