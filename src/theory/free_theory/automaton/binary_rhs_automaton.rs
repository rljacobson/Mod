/*!

A RHS automaton specialized for the two argument case.

ToDo: Put the args inline.

*/

use std::{
  cell::RefCell,
  any::Any,
  rc::Rc
};

use crate::{
  theory::{
    free_theory::{
      automaton::FreeRHSAutomatonInstruction,
      FreeDagNode
    },
    dag_node::MaybeDagNode,
    DagNode,
    RcDagNode,
    RcSymbol,
    RHSAutomaton,
  },
  rc_cell,
  core::{
    VariableInfo,
    substitution::Substitution
  },
  abstractions::RcCell,
};

#[derive(Default)]
pub struct FreeBinaryRHSAutomaton {
  symbol: Option<RcSymbol>,
  instructions: Vec<FreeRHSAutomatonInstruction>,
  sources: [i32; 2],
  destination: i32,
}

impl FreeBinaryRHSAutomaton {
  #[inline(always)]
  fn fill_out_args(&self, matcher: &Substitution, dag_node: &mut dyn DagNode) {
    dag_node.dag_node_members_mut().args.push(matcher.value(0).unwrap());
    dag_node.dag_node_members_mut().args.push(matcher.value(1).unwrap());
  }
}

impl RHSAutomaton for FreeBinaryRHSAutomaton {

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    // Standard processing.
    for instr in &mut self.instructions {
      instr.destination = variable_info.remap_index(instr.destination);
      for source in &mut instr.sources {
        *source = variable_info.remap_index(*source);
      }
    }

    // Make fast copy.
    let instr = self.instructions[0].clone();
    self.symbol = Some(instr.symbol);
    self.sources[0] = instr.sources[0];
    self.sources[1] = instr.sources[1];
    self.destination = instr.destination;
  }

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    let mut new_dag_node = FreeDagNode::new(self.symbol.unwrap().clone());
    self.fill_out_args(matcher, &mut new_dag_node);

    let maybe_dag_node: MaybeDagNode = Some(rc_cell!(new_dag_node));
    matcher.bind(self.destination as i32, maybe_dag_node.clone());

    maybe_dag_node
  }

  fn replace(&mut self, old: RcDagNode, matcher: &mut Substitution) {
    let new_dag_node: FreeDagNode =  FreeDagNode::new(self.symbol.unwrap().clone());

    if let Some(old_node) = old.borrow_mut().as_any_mut().downcast_mut::<FreeDagNode>(){
      let _ = std::mem::replace(old_node, new_dag_node);
      self.fill_out_args(matcher, old_node);
    } else{
      unreachable!("Attempted to swap non free dag node for free dag node. This is a bug.");
    }
  }

}
