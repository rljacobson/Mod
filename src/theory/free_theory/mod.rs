/*!

The free theory: Functions that are not assumed to have additional structure (associativity, commutativity, etc.).

*/

mod automaton;
mod dag_node;
mod free_net;
mod term;
mod remainder;
mod symbol;

pub use automaton::FreeLHSAutomaton;
pub use dag_node::{FreeDagNode, RcFreeDagNode};
pub use free_net::{FreeNet, PatternSet, RcFreeNet};
pub use term::{FreeTerm, RcFreeTerm};
pub use remainder::{FreeRemainder, RcFreeRemainder, FreeRemainderList};
pub use symbol::{FreeSymbol, RcFreeSymbol};


use crate::core::RcSort;
use crate::theory::RcTerm;
use crate::theory::variable::RcVariableTerm;
use super::{LHSAutomaton, Term};


pub type FreeTermOccurrence = FreeOccurrence<RcTerm>;
pub type VariableTermOccurrence = FreeOccurrence<RcVariableTerm>;

struct FreeOccurrence<T: Clone> {
  position : u32,
  arg_index: u32,
  term     : T
}

// These structs are defined in theory/automaton/mod.rs
/*
struct FreeVariable {
  position : u32,
  arg_index: u32,
  var_index: i32,
  sort     : RcSort,
}

struct BoundVariable {
  position : u32,
  arg_index: u32,
  var_index: i32,
}

struct GroundAlien {
  position : u32,
  arg_index: u32,
  alien      : Box<dyn Term>,
}

struct NonGroundAlien {
  position  : u32,
  arg_index : u32,
  automaton : Box<dyn LHSAutomaton>,
}
*/
