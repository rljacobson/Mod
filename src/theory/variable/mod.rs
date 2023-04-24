/*

Variables implement a lot of the same traits as a theory does, but it's technically not a theory.

*/

mod term;
mod dag_node;
mod symbol;

pub use term::{VariableTerm, RcVariableTerm};
pub use dag_node::{VariableDagNode};
