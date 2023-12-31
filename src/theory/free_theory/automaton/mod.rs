mod binary_rhs_automaton;
mod fast_2_rhs_automaton;
mod fast_3_rhs_automaton;
mod lhs_automaton;
mod nullary_rhs_automaton;
mod rhs_automaton;
mod ternary_rhs_automaton;
mod unary_rhs_automaton;

pub use binary_rhs_automaton::FreeBinaryRHSAutomaton;
pub use fast_2_rhs_automaton::FreeFast2RHSAutomaton;
pub use fast_3_rhs_automaton::FreeFast3RHSAutomaton;
pub use lhs_automaton::FreeLHSAutomaton;
pub use nullary_rhs_automaton::FreeNullaryRHSAutomaton;
pub use rhs_automaton::FreeRHSAutomaton;
pub use ternary_rhs_automaton::FreeTernaryRHSAutomaton;
pub use unary_rhs_automaton::FreeUnaryRHSAutomaton;

use crate::theory::RcSymbol;


#[derive(Clone)]
pub(crate) struct FreeRHSAutomatonInstruction {
  pub(crate) symbol:      RcSymbol,
  pub(crate) destination: i32,
  pub(crate) sources:     Vec<i32>,
}
