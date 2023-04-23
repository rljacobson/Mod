/*!

The matcher automaton for the free theory.

*/

use std::rc::Rc;

use crate::abstractions::RcCell;

use crate::{
    core::{RcSort, Substitution},
    theory::{ExtensionInfo, MaybeSubproblem, RcDagNode, RcTerm},
};

pub struct FreeVariable {
    pub position  : u16,
    pub arg_index : u16,
    pub var_index : i32,
    pub sort      : RcSort,
}

pub struct BoundVariable {
    pub position  : u16,
    pub arg_index : u16,
    pub var_index : i32,
}

pub struct GroundAlien {
    pub position  : u16,
    pub arg_index : u16,
    pub alien     : RcTerm,
}

pub struct NonGroundAlien {
    pub position  : u16,
    pub arg_index : u16,
    // TODO: `NonGroundAlien` owns its LHSAutomaton.
    pub automaton : BxLHSAutomaton,
}

pub type RcLHSAutomaton = RcCell<dyn LHSAutomaton>;
pub type BxLHSAutomaton = Box<dyn LHSAutomaton>;

pub trait LHSAutomaton {
    fn match_(
        &mut self,
        subject: RcDagNode,
        solution: &mut Substitution,
        // returned_subproblem: Option<&mut dyn Subproblem>,
        // extension_info: Option<&mut dyn ExtensionInfo>,
    ) -> (bool, MaybeSubproblem);
}

pub(crate) trait RHSAutomaton {}


///	This trait must be derived from for equational theories that generate
///	unification subproblems.
pub(crate) trait UnificationSubproblem {}


//	These traits should be derived from for theories supported by
//	the stack based interpreter.
pub(crate) trait Instruction {}
/// instruction with regular GC handling
pub(crate) trait RegularInstruction {}
/// regular instruction that is not the last instruction in its sequence
pub(crate) trait NonFinalInstruction {}
/// regular ctor that is not the last instruction in its sequence
pub(crate) trait NonFinalConstructor {}
/// regular extor that is not the last instruction in its sequence
pub(crate) trait NonFinalExecutor {}
/// regular instruction that is the final instruction in its sequence
pub(crate) trait FinalInstruction {}
