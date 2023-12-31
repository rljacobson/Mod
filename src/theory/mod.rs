#![allow(unused_imports)]
/*!

Traits that the components of theory must implement.

*/

//
//	These traits must be derived from for all equational theories.
//

// BinarySymbol is a Symbol, AssociativeSymbol is a BinarySymbol
// pub trait Symbol {}		// or
// pub trait BinarySymbol {}	// or
// pub trait AssociativeSymbol {}

pub mod symbol_type;


mod automaton;
mod dag_node;
mod dag_node_flags;
mod subproblem;
mod symbol;
mod term;
// mod associative_symbol;

// Theories
pub mod free_theory;
pub mod term_compiler;
pub mod variable;
// pub mod acu_theory;

use std::rc::Rc;

pub(crate) use automaton::{
  lhs_automaton::{BxLHSAutomaton, LHSAutomaton, RcLHSAutomaton},
  rhs_automaton::{BxRHSAutomaton, RHSAutomaton, RcRHSAutomaton},
};
pub(crate) use dag_node::{AtomicNodeList, DagNode, DagNodeMembers, DagPair, NodeList, RcDagNode};
pub(crate) use dag_node_flags::{DagNodeFlag, DagNodeFlags};
pub(crate) use subproblem::{
  ExtensionInfo,
  MaybeSubproblem,
  RcSubproblem,
  Subproblem,
  SubproblemSequence,
  VariableAbstractionSubproblem,
};
pub(crate) use symbol::{BinarySymbol, RcSymbol, Symbol, SymbolMembers, SymbolSet};
pub(crate) use symbol_type::{BasicSymbolTypes, SymbolType};
pub(crate) use term::{
  find_available_terms,
  index_variables,
  MaybeTerm,
  NodeCache,
  RcTerm,
  Term,
  TermAttribute,
  TermMembers,
  TermSet,
};


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AssociativeSymbolStructure {
  Unstructured,
  // no guarantees
  LimitSort,
  // s_1 <= s & s_2 <= s ===> s_f(s_1, s_2) <= s
  PureSort, // replaces ===> with <===>, taking sort constraints in to account
}


// Todo: Should we use Option<bool>?
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
  Success,
  Failure,
  Undecided, // Unknown
}


impl From<bool> for Outcome {
  fn from(value: bool) -> Self {
    if value {
      Outcome::Success
    } else {
      Outcome::Failure
    }
  }
}
