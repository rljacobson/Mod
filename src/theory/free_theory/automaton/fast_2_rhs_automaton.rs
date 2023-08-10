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

// Stores DAG nodes inline.
struct FreeFast2RHSInstruction {
  pub(crate) symbol     : RcSymbol,
  pub(crate) destination: i32,
  pub(crate) sources    : [i32; 2],
}


#[derive(Default)]
pub struct FreeFast2RHSAutomaton {
  symbol: Option<RcSymbol>,
  instructions: Vec<FreeRHSAutomatonInstruction>,
  fast_instructions: Vec<FreeFast2RHSInstruction>,
  sources: [i32; 2],
  destination: i32,
}

impl FreeFast2RHSAutomaton {
  #[inline(always)]
  fn fill_out_args(&self, instruction: &FreeFast2RHSInstruction, matcher: &Substitution, dag_node: &mut dyn DagNode) {
    dag_node.dag_node_members_mut()
            .args
            .push(matcher.value(instruction.sources[0] as usize).unwrap());
    dag_node.dag_node_members_mut()
            .args
            .push(matcher.value(instruction.sources[1] as usize).unwrap());
  }
}

impl RHSAutomaton for FreeFast2RHSAutomaton {

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
    let instruction_count = self.instructions.len();
    self.fast_instructions.reserve(instruction_count);
    for i in 0..instruction_count {
      let instr = &self.instructions[i];
      let mut f = FreeFast2RHSInstruction {
        symbol: instr.symbol.clone(),

        destination: instr.destination,
        sources: [0, 0],
      };
      // `instr.sources` may have fewer than 2 elmts.
      for (i,  &source) in instr.sources.iter().enumerate(){
        f.sources[i] = source;
      }

      self.fast_instructions.push(f);
    }
  }

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    let mut instruction_iter = self.fast_instructions.iter();
    let mut instruction =
        if let Some(i) = instruction_iter.next() {
          i
        } else {
          return None;
        };

    loop {
      let mut new_dag_node = FreeDagNode::new(instruction.symbol.clone());
      self.fill_out_args(instruction, matcher, &mut new_dag_node);

      let maybe_dag_node: MaybeDagNode = Some(rc_cell!(new_dag_node));
      matcher.bind(instruction.destination, maybe_dag_node.clone());

      if let Some(i) = instruction_iter.next() {
        instruction = i;
      } else {
        return maybe_dag_node
      }
    };

  }

  fn replace(&mut self, old: RcDagNode, matcher: &mut Substitution) {
    let mut instruction_count = 0;

    for instruction in &self.fast_instructions {
      let mut new_dag_node = FreeDagNode::new(instruction.symbol.clone());
      self.fill_out_args(instruction, matcher, &mut new_dag_node);

      instruction_count += 1;
      // ToDo: Why isn't the last one bound?
      if instruction_count == self.fast_instructions.len() {
        if let Some(old_node) = old.borrow_mut().as_any_mut().downcast_mut::<FreeDagNode>(){
          let _ = std::mem::replace(old_node, new_dag_node);
        } else{
          unreachable!("Attempted to swap non free dag node for free dag node. This is a bug.");
        }
        break;
      }

      matcher.bind(instruction.destination, Some(rc_cell!(new_dag_node)));
    }

  }

}
