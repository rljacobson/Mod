/*!

The free theory: Functions that are not assumed to have additional structure (associativity, commutativity, etc.).

*/

mod automaton;
mod dag_node;
mod free_net;
mod term;
mod remainder;
mod symbol;

use std::cell::RefCell;
pub use automaton::FreeLHSAutomaton;
pub use dag_node::{FreeDagNode, RcFreeDagNode};
pub use free_net::{FreeNet, PatternSet, RcFreeNet};
pub use term::{FreeTerm, RcFreeTerm};
pub use remainder::{FreeRemainder, RcFreeRemainder, FreeRemainderList};
pub use symbol::{FreeSymbol, RcFreeSymbol};


use crate::core::RcSort;
use crate::theory::{RcLHSAutomaton, RcTerm};
use crate::theory::variable::RcVariableTerm;
use super::{LHSAutomaton, Term};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FreeOccurrence {
  position : u32,
  arg_index: u32,
  term     : *mut dyn Term
}

impl FreeOccurrence {
  pub fn dereference_term<T: 'static>(&self) -> &mut T {
    let term: &mut dyn Term = unsafe{ &mut *self.term };

    if let Some(term) = term.as_any_mut().downcast_mut::<T>() {
      term
    } else {
      unreachable!("Could not dereference as FreeTerm. This is a bug.")
    }
  }
}

// These structs are specific to the free theory. The ACU theory has its own version.
#[derive(Clone, Eq, PartialEq)]
pub struct FreeVariable {
  position : u32,
  arg_index: u32,
  var_index: i32,
  sort     : RcSort,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BoundVariable {
  position : u32,
  arg_index: u32,
  var_index: i32,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct GroundAlien {
  position : u32,
  arg_index: u32,
  alien    : *mut dyn Term,
}


impl GroundAlien {
  pub fn dereference_term<T: 'static>(&self) -> &mut T {
    let term: &mut dyn Term = unsafe{ &mut *self.alien };

    if let Some(term) = term.as_any_mut().downcast_mut::<T>() {
      term
    } else {
      unreachable!("Could not dereference as FreeTerm. This is a bug.")
    }
  }
}


#[derive(Clone, PartialEq)]
pub struct NonGroundAlien {
  position  : u32,
  arg_index : u32,
  automaton : RcLHSAutomaton //RefCell<dyn LHSAutomaton>,
}

