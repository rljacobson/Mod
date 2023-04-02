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


pub(crate) mod symbol;
pub(crate) mod dag_node;
pub(crate) mod term;
pub(crate) mod subproblem;
pub(crate) mod free_theory;
pub(crate) mod automaton;
// mod associative_symbol;
// mod acu_theory;

use std::rc::Rc;

pub(crate) use super::{
  theory::{
    subproblem::{
      ExtensionInfo,
      Subproblem,
      MaybeSubproblem,
      VariableAbstractionSubproblem,
      SubproblemSequence
    },
    // associative_symbol::AssociativeSymbolStructure,
    term::{
      Term,
      RcTerm,
      Flags
    },
    dag_node::{
      DagNode,
      DagPair,
      RcDagNode
    },
    symbol::{
      Symbol,
      RcSymbol,
      BinarySymbol
    },
  }
};
use crate::core::Substitution;


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

