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


mod symbol;
mod dag_node;
mod term;
mod subproblem;
mod free_theory;
// mod associative_symbol;
// mod acu_theory;

pub(crate) use crate::{
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
      BinarySymbol
    },
  }
};
use crate::Substitution;


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


pub trait LhsAutomaton {
  fn match_(
    &mut self,
    subject            : RcDagNode,
    solution           : &mut Substitution,
    // returned_subproblem: Option<&mut dyn Subproblem>,
    extension_info     : Option<&mut dyn ExtensionInfo>
  ) -> (bool, MaybeSubproblem);
}

pub(crate) trait RhsAutomaton {}

//
//	This trait must be derived from for equational theories that generate
//	unification subproblems.
//
pub(crate) trait UnificationSubproblem {}
//	These traits can be should be derived from for theories supported by
//	the stack based interpreter.
//
pub(crate) trait Instruction {}
pub(crate) trait RegularInstruction {}  // instruction with regular GC handling
pub(crate) trait NonFinalInstruction {}  // regular instruction that is not the last instruction in its sequence
pub(crate) trait NonFinalCtor {}  // regular ctor that is not the last instruction in its sequence
pub(crate) trait NonFinalExtor {}  // regular extor that is not the last instruction in its sequence
pub(crate) trait FinalInstruction {}  // regular instruction that is the final instruction in its sequence


