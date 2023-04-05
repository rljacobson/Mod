/*!

The free theory: Functions that are not assumed to have additional structure (associativity, commutativity, etc.).

*/

mod dag_node;
mod free_term;
mod free_net;
mod symbol;
mod remainder;
mod automaton;


pub use free_term::FreeTerm;
pub use free_net::{FreeNet, RcFreeNet, PatternSet};
pub use remainder::{FreeRemainder, RcFreeRemainder};
pub use symbol::{FreeSymbol, RcFreeSymbol};
pub use dag_node::{FreeDagNode, RcFreeDagNode};
pub use automaton::FreeLHSAutomaton;
