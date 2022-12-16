/*!



 */


use crate::{
  theory::{
    DagNode,
    LhsAutomaton,
    RcDagNode,
    Term,
    acu_theory::RedBlackTree,
    subproblem::Subproblem
  },
  Sort,
  Substitution,
  RewritingContext,
};
use crate::sort::RcSort;

pub type MaybeSubproblem = Option<Box<dyn Subproblem>>;

pub struct ACUSubproblem {
  solved: bool
}

/// The following are part of DelayedSubproblem in Maude, but ACUSubproblem is the only implementor.
impl ACUSubproblem {

  fn delayed_solve(&mut self, _find_first: bool, _context: &mut RewritingContext) -> bool {
    // Todo: Implement ACUSubproblem::delayed_solve(..).
    return false;
  }

  fn is_solved(&self) -> bool{
    return self.solved;
  }

  fn set_solved(&mut self, solved: bool) {
    self.solved = solved;
  }


}

impl Subproblem for ACUSubproblem {
  fn solve(&mut self, find_first: bool, context: &mut RewritingContext) -> bool {
    // Todo: Implement ACUSubproblem::solve(..).
    false
  }
}

pub struct ACULazySubproblem<'a> {
  pub(crate) subject      : &'a RedBlackTree,
  pub(crate) current      : &'a mut RedBlackTree,
  pub(crate) solution     : &'a mut Substitution,
  pub(crate) lhs_automaton: &'a mut LhsAutomaton,
  pub(crate) term         : Option<&'a dyn Term>,
  pub(crate) index        : u32,
  pub(crate) sort         : RcSort
}


impl<'a> Subproblem for ACULazySubproblem<'a> {
  fn solve(&mut self, find_first: bool, context: &mut RewritingContext) -> bool {
    // Todo: Implement ACULazySubproblem::solve(..).
    false
  }
}
