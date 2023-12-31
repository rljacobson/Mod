/*!




*/

use crate::{
  core::{
    substitution::{MaybeDagNode, Substitution},
    VariableInfo,
  },
  theory::{BxRHSAutomaton, DagNode, RHSAutomaton, RcDagNode, RcRHSAutomaton},
};

#[derive(Default)]
pub struct RHSBuilder {
  // TODO: I think `RHSAutomaton` are single owner. Use `BxRHSAutomaton`.
  automata:       Vec<BxRHSAutomaton>,
  last_automaton: Option<BxRHSAutomaton>,
}

impl RHSBuilder {
  pub fn new() -> RHSBuilder {
    RHSBuilder {
      automata:       Vec::new(),
      last_automaton: None,
    }
  }

  pub fn add_rhs_automaton(&mut self, automaton: BxRHSAutomaton) {
    if let Some(last_automaton) = self.last_automaton.take() {
      self.automata.push(last_automaton);
    }
    self.last_automaton = Some(automaton);
  }

  pub fn is_empty(&self) -> bool {
    self.last_automaton.is_none()
  }

  pub fn remap_indices(&mut self, variable_info: &mut VariableInfo) {
    for automaton in self.automata.iter_mut() {
      automaton.remap_indices(variable_info);
    }
    if let Some(last_automaton) = &mut self.last_automaton {
      last_automaton.remap_indices(variable_info);
    }
  }

  // TODO: `record_info` requires `StackMachineRhsCompiler` to be implemented.
  /*
    pub fn record_info(&self, compiler: &mut StackMachineRhsCompiler) -> bool {
      for automaton in self.automata.iter() {
        if !automaton.record_info(compiler) {
          return false;
        }
      }
      match &self.last_automaton {
        Some(last_automaton) => last_automaton.record_info(compiler),
        None => true,
      }
    }
  */

  pub fn construct(&self, matcher: &mut Substitution) -> MaybeDagNode {
    for automaton in self.automata.iter() {
      automaton.construct(matcher);
    }
    if let Some(last_automaton) = &self.last_automaton {
      last_automaton.construct(matcher)
    } else {
      None
    }
  }

  pub fn safe_construct(&self, matcher: &mut Substitution) {
    for automaton in self.automata.iter() {
      automaton.construct(matcher);
    }
    if let Some(last_automaton) = self.last_automaton.as_ref() {
      last_automaton.construct(matcher);
    }
  }

  pub fn replace(&mut self, old: RcDagNode, matcher: &mut Substitution) {
    for automaton in self.automata.iter() {
      automaton.construct(matcher);
    }
    if let Some(last_automaton) = self.last_automaton.as_mut() {
      last_automaton.replace(old, matcher);
    }
  }
}
