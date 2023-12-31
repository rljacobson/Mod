/*

Variables implement a lot of the same traits as a theory does, but it's technically not a theory.

*/

mod automaton;
mod dag_node;
mod symbol;
mod term;

pub use automaton::VariableLHSAutomaton;
pub use dag_node::VariableDagNode;
pub use symbol::{RcVariableSymbol, VariableSymbol};
pub use term::{RcVariableTerm, VariableTerm};
