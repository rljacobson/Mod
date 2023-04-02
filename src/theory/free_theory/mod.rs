/*!

The free theory: Functions that are not assumed to have additional structure (associativity, commutativity, etc.).

*/

mod free_net;
mod symbol;
mod remainder;

pub use free_net::{FreeNet, RcFreeNet, PatternSet};
pub use remainder::{FreeRemainder, RcFreeRemainder};
pub use symbol::{FreeSymbol, RcFreeSymbol};
