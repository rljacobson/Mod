/*!

The free theory: Functions that are not assumed to have additional structure (associativity, commutativity, etc.).

*/

mod automaton;
mod dag_node;
mod free_net;
mod free_term;
mod remainder;
mod symbol;

pub use automaton::FreeLHSAutomaton;
pub use dag_node::{FreeDagNode, RcFreeDagNode};
pub use free_net::{FreeNet, PatternSet, RcFreeNet};
pub use free_term::FreeTerm;
pub use remainder::{FreeRemainder, RcFreeRemainder, FreeRemainderList};
pub use symbol::{FreeSymbol, RcFreeSymbol};


use crate::core::RcSort;
use super::{LHSAutomaton, Term};

struct FreeVariable {
  position : i16,
  argIndex : i16,
  varIndex : i32,
  sort     : RcSort,
}

struct BoundVariable {
  position : i16,
  argIndex : i16,
  varIndex : i32,
}

struct GroundAlien {
  position : i16,
  argIndex : i16,
  ali      : Box<dyn Term>,
}

struct NonGroundAlien {
  position  : i16,
  argIndex  : i16,
  automaton : Box<dyn LHSAutomaton>,
}
