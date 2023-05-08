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
mod dag_node;
mod term;
mod subproblem;
mod automaton;
mod associative_symbol;
pub mod free_theory;
pub mod variable;
mod dag_node_flags;
// mod acu_theory;

use std::rc::Rc;


pub(crate) use subproblem::{
  ExtensionInfo,
  Subproblem,
  MaybeSubproblem,
  VariableAbstractionSubproblem,
  SubproblemSequence
};
pub(crate) use associative_symbol::{
  AssociativeSymbolStructure,

};
pub(crate) use term::{
  Term,
  RcTerm,
  TermFlags,
  TermMembers
};
pub(crate) use dag_node_flags::{
  DagNodeFlag,
  DagNodeFlags,
};
pub(crate) use dag_node::{
  DagNode,
  DagPair,
  RcDagNode,
  NodeList,
  AtomicNodeList,
};
pub(crate) use symbol::{
  Symbol,
  RcSymbol,
  BinarySymbol
};
pub(crate) use symbol_type::{
  SymbolType,
  BasicSymbolTypes
};
pub(crate) use automaton::{
  LHSAutomaton,
  BxLHSAutomaton,
  RcLHSAutomaton
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

