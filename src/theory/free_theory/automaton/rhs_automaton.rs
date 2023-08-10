/*!

The RHS automaton for the free theory has six variations specialized for arities 0-3:

    * FreeFast3RHSAutomaton
    * FreeFast2RHSAutomaton
    * FreeTernaryRHSAutomaton
    * FreeBinaryRHSAutomaton
    * FreeUnaryRHSAutomaton
    * FreeNullaryRHSAutomaton

 */

use std::{
  cell::RefCell,
  rc::Rc
};

use crate::{
  abstractions::{
    RcCell,
    rc_cell
  },
  core::{
    substitution::Substitution,
    VariableInfo,
  },
  theory::{
    dag_node::MaybeDagNode,
    free_theory::{
      automaton::{
        FreeRHSAutomatonInstruction,
        FreeNullaryRHSAutomaton,
        FreeUnaryRHSAutomaton,
        FreeBinaryRHSAutomaton,
        FreeFast2RHSAutomaton,
        FreeFast3RHSAutomaton,
        FreeTernaryRHSAutomaton
      },
      FreeDagNode
    },
    DagNode,
    RcDagNode,
    RcSymbol,
    RHSAutomaton,
  }
};


#[derive(Default)]
pub struct FreeRHSAutomaton {
  instructions: Vec<FreeRHSAutomatonInstruction>
}

impl FreeRHSAutomaton {

  fn new() -> Self {
    Self::default()
  }

  pub fn with_arity_and_free_variable_count(max_arity: u32, free_variable_count: u32) -> Box<dyn RHSAutomaton> {

    if max_arity > 3 {
      Box::new(FreeRHSAutomaton::new()) // general case
    }
    else {
      // We have six faster RHS automata for low arity cases.
      if free_variable_count > 1 {
        // Multiple low arity symbol cases.
        if max_arity == 3 {
          Box::new(FreeFast3RHSAutomaton::default()) // all dag nodes padded to 3 args
        } else {
          Box::new(FreeFast2RHSAutomaton::default()) // all dag nodes padded to 2 args
        }
      } else {
        // Single low arity symbol cases.
        if max_arity > 1 {
          if max_arity == 3 {
            Box::new(FreeTernaryRHSAutomaton::default())
          } else {
            Box::new(FreeBinaryRHSAutomaton::default())
          }
        } else {
          if max_arity == 1 {
            Box::new(FreeUnaryRHSAutomaton::default())
          } else {
            Box::new(FreeNullaryRHSAutomaton::default())
          }
        }
      }
    }
  }

  pub fn add_free(&mut self, symbol: RcSymbol, destination: i32, sources: &Vec<i32>) {
    let new_instruction = FreeRHSAutomatonInstruction{
      symbol,
      destination,
      sources: sources.clone(),
    };

    self.instructions.push(new_instruction);
  }

  fn fill_out_args(&self, instr: &FreeRHSAutomatonInstruction, matcher: &mut Substitution, dag_node: &mut dyn DagNode) {
    let arg_count = dag_node.symbol().arity();
    if arg_count != 0 {
      for (j, mut arg) in dag_node.iter_args().enumerate() {
        let mut new_arg = matcher.value(instr.sources[j] as usize);
        arg.0 = new_arg.unwrap().0;
      }
    }
  }
}


impl RHSAutomaton for FreeRHSAutomaton {

  fn as_any(&self) ->  &dyn std::any::Any {
    self
  }

  fn as_any_mut(&mut self) ->  &mut dyn std::any::Any {
    self
  }


  fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    for instr in &mut self.instructions {
      instr.destination = variable_info.remap_index(instr.destination);
      for source in &mut instr.sources {
        *source = variable_info.remap_index(*source);
      }
    }
  }

  fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    let mut new_dag_node: MaybeDagNode = None;

    for i in self.instructions.iter() {
      let mut naked_dag_node: FreeDagNode = FreeDagNode::new(i.symbol.clone());
      self.fill_out_args(i, matcher, &mut naked_dag_node);

      new_dag_node = Some(rc_cell!(naked_dag_node));
      matcher.bind(i.destination as i32, new_dag_node.clone());
    }

    new_dag_node
  }

  fn replace(&mut self, mut old: RcDagNode, matcher: &mut Substitution) {
    let nr_instructions = self.instructions.len();

    for instruction in &self.instructions[..nr_instructions-1] {
      let mut new_dag_node = FreeDagNode::new(instruction.symbol.clone());
      self.fill_out_args(instruction, matcher, &mut new_dag_node);

      let new_dag_node: RcDagNode = rc_cell!(new_dag_node);
      matcher.bind(instruction.destination as i32, Some(new_dag_node));
    }

    let instruction = &self.instructions[nr_instructions-1];

    if let Some(old_node) = old.borrow_mut().as_any_mut().downcast_mut::<FreeDagNode>(){
      let new_dag_node =  FreeDagNode::new(instruction. symbol.clone());
      let _ = std::mem::replace(old_node, new_dag_node);

      self.fill_out_args(instruction, matcher, old_node);
    } else{
      unreachable!("Attempted to swap non free dag node for free dag node. This is a bug.");
    }

  }
}
