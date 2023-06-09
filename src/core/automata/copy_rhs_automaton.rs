/*!

Right hand side automata that make copies of bindings in the substitution.


*/


use std::any::Any;
use pratt::{Channel, log};
use crate::{
  core::{
    VariableInfo,
    substitution::{
      MaybeDagNode,
      Substitution
    }
  },
  theory::{
    RcDagNode,
    RHSAutomaton
  }
};

pub struct CopyRHSAutomaton {
  original_index: i32,
  copy_index: i32,
}


impl CopyRHSAutomaton {
  pub fn new(original_index: i32, copy_index: i32) -> Self {
    // TODO: Are these indices necessarily positive?
    Self {
      original_index,
      copy_index,
    }
  }
}


impl RHSAutomaton for CopyRHSAutomaton {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    self.original_index = variable_info.remap_index(self.original_index);
    self.copy_index = variable_info.remap_index(self.copy_index);
  }

  /*
  fn record_info(&self, compiler: &mut StackMachineRhsCompiler) -> bool {
    let mut sources = Vec::new();
    sources.push(self.original_index);
    compiler.record_function_eval(0, self.copy_index, sources);
    true
  }
  */

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    let orig = matcher.value(self.original_index as usize);
    if let Some(orig_dag_node) = orig{
      log(Channel::Debug, 2, format!("CopyRhsAutomaton::construct {}", orig_dag_node.borrow()).as_str());

      let mut new_dag_node = orig_dag_node.borrow_mut().copy_eager_upto_reduced();
      orig_dag_node.borrow_mut().clear_copied_rc();
      matcher.bind(self.copy_index as i32, new_dag_node.clone());
      new_dag_node
    } else {
      unreachable!("No DagNode for original index. This is a bug.");
    }
  }

  fn replace(&mut self, old: RcDagNode, matcher: &mut Substitution) {
    let orig = matcher.value(self.original_index as usize);

    if let Some(orig_dag_node) = orig{
      let mut new_dag_node = orig_dag_node.borrow_mut().copy_eager_upto_reduced();
      orig_dag_node.borrow_mut().clear_copied_rc();

      if let Some(new_dag_node) = new_dag_node {
        new_dag_node.borrow_mut().overwrite_with_clone(old);
      }
    } else {
      unreachable!("No DagNode for original index. This is a bug.");
    }
  }

}

