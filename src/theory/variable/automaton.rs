use crate::{
  core::{
    sort::RcSort,
    substitution::Substitution
  },
  theory::{
    LHSAutomaton,
    MaybeSubproblem,
    RcDagNode
  },
};

pub struct VariableLHSAutomaton {
  index: i32, // -1 = None
  sort : RcSort,
  copy_to_avoid_overwriting: bool
}

impl VariableLHSAutomaton {
  pub fn new(index: i32, sort: RcSort, copy_to_avoid_overwriting: bool) -> Self {
    VariableLHSAutomaton {
      index,
      sort,
      copy_to_avoid_overwriting
    }
  }
}


impl LHSAutomaton for VariableLHSAutomaton {
  fn match_(&mut self, subject: RcDagNode, solution: &mut Substitution) -> (bool, MaybeSubproblem) {
    self.match_variable(
      subject,
      self.index,
      self.sort.clone(),
      self.copy_to_avoid_overwriting,
      solution
    )
  }
}
