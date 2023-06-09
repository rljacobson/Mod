use crate::{
  abstractions::RcCell,
  core::{
    VariableInfo,
    substitution::Substitution
  },
  theory::{
    dag_node::MaybeDagNode,
    RcDagNode
  },
};


pub type RcRHSAutomaton = RcCell<dyn RHSAutomaton>;
pub type BxRHSAutomaton = Box<dyn RHSAutomaton>;

pub trait RHSAutomaton {
  fn as_any(&self) -> &dyn std::any::Any;
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;


  fn remap_indices(&mut self, variable_info: &mut VariableInfo);
  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode;
  fn replace(&mut self, old: RcDagNode, matcher: &mut Substitution);

  // TODO: `StackMachineRhsCompiler` is not yet implemented.
  /*
  fn record_info(&self, _compiler: &mut StackMachineRhsCompiler) -> bool {
    false
  }
  */
}
