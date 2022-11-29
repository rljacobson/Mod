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
mod acu_theory;
mod dag_node;
mod term;
mod subproblem;
mod associative_symbol;

pub(crate) use crate::{
  theory::{
    subproblem::{
      ExtensionInfo,
      Subproblem
    },
    associative_symbol::AssociativeSymbolStructure,
    term::Term,
    dag_node::{DagNode, DagPair},
    symbol::Symbol
  }
};
use crate::substitution::Substitution;


pub trait LhsAutomaton {
  fn match_(
    &self,
    subject            : &dyn DagNode,
    solution           : &Substitution,
    returned_subproblem: Option<&mut dyn Subproblem>,
    extension_info     : &dyn ExtensionInfo
  ) -> bool;
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


