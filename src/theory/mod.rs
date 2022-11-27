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

use crate::theory::dag_node::DagNode;

pub trait LhsAutomaton {
  fn match_(
    &self,
    subject            : *DagNode,
    solution           : &Substitution,
    returned_subproblem: *&Subproblem,
    extension_info     : *ExtensionInfo
  ) -> bool;
}

pub trait RhsAutomaton {}
//
//	These traits must be derived from for equational theories that
//	need to generate matching or unification subproblems  or
//	pass back extension information respectively.
//
pub trait Subproblem {}
pub trait ExtensionInfo {}
//
//	This trait must be derived from for equational theories that generate
//	unification subproblems.
//
pub trait UnificationSubproblem {}
//
//	This trait can be derived from for equational theories that want to
//	delay and batch	subproblems in the hope of reducing the search space.
//
pub trait DelayedSubproblem {}
//
//	These traits can be should be derived from for theories supported by
//	the stack based interpreter.
//
pub trait Instruction {}
pub trait RegularInstruction {}  // instruction with regular GC handling
pub trait NonFinalInstruction {}  // regular instruction that is not the last instruction in its sequence
pub trait NonFinalCtor {}  // regular ctor that is not the last instruction in its sequence
pub trait NonFinalExtor {}  // regular extor that is not the last instruction in its sequence
pub trait FinalInstruction {}  // regular instruction that is the final instruction in its sequence


