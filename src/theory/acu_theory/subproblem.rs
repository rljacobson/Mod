/*!

 The structures described here correspond roughly to Fig. 1 of [Eker 1995].

 [Eker 1995]: Eker, Steven. “Associative-Commutative Matching Via Bipartite Graph Matching.” Comput. J. 38 (1995): 381-399.




 */

use diophantine::DiophantineSystem;

use crate::{
  theory::{
    LhsAutomaton,
    RcDagNode,
    Term,
    acu_theory::RedBlackTree,
    subproblem::Subproblem
  },
  Substitution,
  RewritingContext,
  local_bindings::LocalBindings
};
use crate::sort::RcSort;

use super::extension_info::ACUExtensionInfo;


/// Private TopVariable type for subproblem.
struct TopVariable {
  index: u32,
  multiplicity: u32,
  lower_bound: u32,
  upper_bound: u32,
  sort: RcSort
}


struct Edge {
  target: u32,
  difference: Box<LocalBindings>,
  subproblem: Box<dyn Subproblem>
}


struct PatternNode {
  multiplicity : u32,
  edges        : Vec<Edge>,
  selected_edge: u32
}

impl PatternNode {
  pub fn solve(&mut self, find_first: bool, solutuion: &mut RewritingContext, current_multiplicity: &mut Vec<u32>) -> bool {
    true
  }
}


pub struct ACUSubproblem {
  subject: RcDagNode,
  extension_info: Option<ACUExtensionInfo>,

  solved: bool,

  current_multiplicity: Vec<u32>,
  top_variables: Vec<TopVariable>,
  pattern_nodes: Vec<PatternNode>, // Todo: Implement PatternNode

  // Structures needed for building and solving diophantine problem
  system: Option<DiophantineSystem>, // ToDo: Implement DiophantineSystem

  /// For efficiency, the set of variable bindings at each stage in the recursion in both simplify and build_hierarchy
  /// can be tracked by a single global array indexed by small integers representing variables.
  variable_map: Vec<u32>,
  subject_map: Vec<u32>,
  after_multiplicity: Vec<u32>,
}

/// The following are part of DelayedSubproblem in Maude, but ACUSubproblem is the only implementor.
impl ACUSubproblem {

  pub fn delayed_solve(&mut self, _find_first: bool, _context: &mut RewritingContext) -> bool {
    // Todo: Implement ACUSubproblem::delayed_solve(..).
    return false;
  }

  #[inline(always)]
  pub fn is_solved(&self) -> bool{
    return self.solved;
  }

  #[inline(always)]
  pub fn set_solved(&mut self, solved: bool) {
    self.solved = solved;
  }

  #[inline(always)]
  pub fn no_patterns(&self) -> bool{
    self.pattern_nodes.is_empty()
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
  pub(crate) lhs_automaton: &'a mut dyn LhsAutomaton,
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
