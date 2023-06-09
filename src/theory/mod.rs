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


mod symbol;
mod dag_node_flags;
mod dag_node;
mod term;
mod subproblem;
mod automaton;
mod associative_symbol;

// Theories
pub mod free_theory;
pub mod variable;
pub mod term_compiler;
// pub mod acu_theory;

use std::rc::Rc;


pub(crate) use subproblem::{
  ExtensionInfo,
  Subproblem,
  RcSubproblem,
  MaybeSubproblem,
  VariableAbstractionSubproblem,
  SubproblemSequence
};
pub(crate) use associative_symbol::{
  AssociativeSymbolStructure,

};
pub(crate) use term::{
  find_available_terms,
  index_variables,
  NodeCache,
  RcTerm,
  Term,
  TermFlags,
  TermMembers,
  TermSet,
  MaybeTerm,
};
pub(crate) use dag_node_flags::{
  DagNodeFlag,
  DagNodeFlags,
};
pub(crate) use dag_node::{
  AtomicNodeList,
  DagNode,
  DagNodeMembers,
  DagPair,
  NodeList,
  RcDagNode,
};
pub(crate) use symbol::{
  Symbol,
  SymbolMembers,
  SymbolSet,
  RcSymbol,
  BinarySymbol
};
pub(crate) use symbol_type::{
  SymbolType,
  BasicSymbolTypes
};
pub(crate) use automaton::{
  lhs_automaton::{
    LHSAutomaton,
    BxLHSAutomaton,
    RcLHSAutomaton
  },
  rhs_automaton::{
    RHSAutomaton,
    RcRHSAutomaton,
    BxRHSAutomaton
  }
};



// Todo: Should we use Option<bool>?
pub enum Outcome {
  Success,
  Failure,
  Undecided // Unknown
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

