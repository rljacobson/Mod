/*!

A RHS automaton specialized for the single argument case.

ToDo: Put the arg inline.

*/

use std::{any::Any, cell::RefCell, rc::Rc};

use crate::{
  abstractions::RcCell,
  core::{substitution::Substitution, VariableInfo},
  rc_cell,
  theory::{
    dag_node::MaybeDagNode,
    free_theory::{automaton::FreeRHSAutomatonInstruction, FreeDagNode},
    DagNode,
    RHSAutomaton,
    RcDagNode,
    RcSymbol,
  },
};

#[derive(Default)]
pub struct FreeUnaryRHSAutomaton {
  symbol:       Option<RcSymbol>,
  instructions: Vec<FreeRHSAutomatonInstruction>,
  source:       i32,
  destination:  i32,
}

impl RHSAutomaton for FreeUnaryRHSAutomaton {
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
    self.source = instr.sources[0];
    self.destination = instr.destination;
  }

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    let new_dag_node: RcDagNode = rc_cell!(FreeDagNode::new(self.symbol.unwrap().clone()));
    matcher.bind(self.destination as i32, Some(new_dag_node.clone()));
    new_dag_node
      .borrow_mut()
      .dag_node_members_mut()
      .args
      .push(matcher.value(self.source as usize).unwrap());

    Some(new_dag_node)
  }

  fn replace(&mut self, old: RcDagNode, matcher: &mut Substitution) {
    let new_dag_node: FreeDagNode = FreeDagNode::new(self.symbol.unwrap().clone());

    if let Some(old_node) = old.borrow_mut().as_any_mut().downcast_mut::<FreeDagNode>() {
      let _ = std::mem::replace(old_node, new_dag_node);
      old_node
        .dag_node_members_mut()
        .args
        .push(matcher.value(self.source as usize).unwrap());
    } else {
      unreachable!("Attempted to swap non free dag node for free dag node. This is a bug.");
    }
  }
}
