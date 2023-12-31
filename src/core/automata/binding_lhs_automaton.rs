/*!

Class for left hand side automata that just bind a variable and call another lhs automaton to do the
real work.

*/

use crate::{
  core::substitution::Substitution,
  theory::{LHSAutomaton, MaybeSubproblem, RcDagNode, RcLHSAutomaton},
};


pub(crate) struct BindingLHSAutomaton {
  variable_index:    i32,
  real_lhs_automata: RcLHSAutomaton,
}

impl BindingLHSAutomaton {
  pub fn new(variable_index: i32, real_lhs_automata: RcLHSAutomaton) -> Self {
    BindingLHSAutomaton {
      variable_index,
      real_lhs_automata,
    }
  }
}


impl LHSAutomaton for BindingLHSAutomaton {
  fn match_(
    &mut self,
    subject: RcDagNode,
    solution: &mut Substitution,
    // extension_info: Option<&mut dyn ExtensionInfo>,
  ) -> (bool, MaybeSubproblem) {
    let (matched, maybe_subproblem) = self.real_lhs_automata.borrow_mut().match_(subject.clone(), solution);
    if matched {
      solution.bind(self.variable_index, Some(subject));
      return (matched, maybe_subproblem);
    }
    return (false, None);
  }
}
