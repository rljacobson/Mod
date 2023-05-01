/*!

The matcher automaton for the free theory.

*/

use std::rc::Rc;

use crate::abstractions::RcCell;

use crate::{
    core::{RcSort, Substitution},
    theory::{ExtensionInfo, MaybeSubproblem, RcDagNode, RcTerm},
};
use crate::theory::{Outcome, Term};
/*
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

pub struct GroundAlien<T: Term> {
    pub position  : u16,
    pub arg_index : u16,
    pub alien     : RcCell<T>,
}

pub struct NonGroundAlien {
    pub position  : u16,
    pub arg_index : u16,
    // TODO: `NonGroundAlien` owns its LHSAutomaton.
    pub automaton : RcLHSAutomaton,
}
*/
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


    // In Maude this is a method on DagNode.
    fn match_variable(
        &self,
        dag_node: RcDagNode,
        index: i32,
        sort: RcSort,
        copy_to_avoid_overwriting: bool,
        solution: &mut Substitution,
        // extension_info: Option<&mut dyn ExtensionInfo>,
    ) -> (bool, MaybeSubproblem)
    {
        // if let Some(ext_info) = extension_info {
        //   return self.match_variable_with_extension(
        //     index,
        //     sort,
        //     solution,
        //     returned_subproblem,
        //     ext_info,
        //   );
        // }

        if let Some(d) = solution.value(index) {
            if dag_node.borrow().compare(d.as_ref()).is_eq() {
                return (true, None);
            }
        }
        else if let (Outcome::Success, subproblem) = dag_node.borrow_mut().check_sort(sort) {
            let new_dag_node = if copy_to_avoid_overwriting {
                dag_node.borrow().shallow_copy()
            } else {
                dag_node.clone()
            };
            solution.bind(index, Some(new_dag_node));
            return (true, subproblem);
        }

        (false, None)
    }


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
