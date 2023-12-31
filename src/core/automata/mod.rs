/*!

Various automata not specific to a single particular theory.


*/

mod binding_lhs_automaton;
mod copy_rhs_automaton;
mod rhs_builder;
mod trivial_rhs_automata;

pub(crate) use binding_lhs_automaton::BindingLHSAutomaton;
pub(crate) use copy_rhs_automaton::CopyRHSAutomaton;
pub(crate) use rhs_builder::RHSBuilder;
pub(crate) use trivial_rhs_automata::TrivialRHSAutomaton;
