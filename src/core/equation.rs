use crate::theory::{
  RcTerm,
  automaton::RcLhsAutomaton
};



pub struct Equation {
  label: u32,
  lhs: RcTerm,
  lhsAutomaton: RcLhsAutomaton,
  // lhsDag: DagRoot,  // For unification
  // condition: Vec<ConditionFragment>,
}
