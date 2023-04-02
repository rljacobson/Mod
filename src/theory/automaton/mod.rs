/*!

The matcher automaton for the free theory.

*/

use std::rc::Rc;

use crate::{
  core::{RcSort, Substitution},
  theory::{
    RcTerm,
    RcDagNode,
    ExtensionInfo,
    MaybeSubproblem
  },
};



pub struct FreeVariable {
  position: u16,
  argIndex: u16,
  varIndex: i32,
  sort: RcSort,
}

pub struct BoundVariable {
  position: u16,
  argIndex: u16,
  varIndex: i32,
}

pub struct GroundAlien {
  position: u16,
  argIndex: u16,
  alien: RcTerm,
}

pub struct NonGroundAlien {
  position: u16,
  argIndex: u16,
  automaton: RcLhsAutomaton,
}


pub type RcLhsAutomaton = Rc<dyn LhsAutomaton>;

pub trait LhsAutomaton {
  fn match_(
    &mut self,
    subject       : RcDagNode,
    solution      : &mut Substitution,
    // returned_subproblem: Option<&mut dyn Subproblem>,
    extension_info: Option<&mut dyn ExtensionInfo>
  ) -> (bool, MaybeSubproblem);
}

pub(crate) trait RhsAutomaton {}

//
//	This trait must be derived from for equational theories that generate
//	unification subproblems.
//
pub(crate) trait UnificationSubproblem {}

//
//	These traits can be should be derived from for theories supported by
//	the stack based interpreter.
//
pub(crate) trait Instruction {}
/// instruction with regular GC handling
pub(crate) trait RegularInstruction {}
/// regular instruction that is not the last instruction in its sequence
pub(crate) trait NonFinalInstruction {}
/// regular ctor that is not the last instruction in its sequence
pub(crate) trait NonFinalCtor {}
/// regular extor that is not the last instruction in its sequence
pub(crate) trait NonFinalExtor {}
/// regular instruction that is the final instruction in its sequence
pub(crate) trait FinalInstruction {}

