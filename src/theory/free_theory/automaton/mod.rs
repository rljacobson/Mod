mod lhs_automaton;
mod rhs_automaton;
mod free_nullary_rhs_automaton;
mod free_unary_rhs_automaton;

pub use lhs_automaton::{
  FreeLHSAutomaton
};

pub use rhs_automaton::{
  FreeRHSAutomaton
};

pub use free_nullary_rhs_automaton::{
  FreeNullaryRHSAutomaton
};

pub use free_unary_rhs_automaton::{
  FreeUnaryRHSAutomaton
};
use crate::theory::RcSymbol;


#[derive(Clone)]
pub(crate) struct FreeRHSAutomatonInstruction {
  pub(crate) symbol     : RcSymbol,
  pub(crate) destination: i32,
  pub(crate) sources    : Vec<i32>,
}

