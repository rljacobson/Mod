/*!

The free theory: Functions that are not assumed to have additional structure (associativity, commutativity, etc.).

*/

mod automaton;
mod dag_node;
mod free_net;
mod remainder;
mod symbol;
mod term;

use std::cell::RefCell;

pub use automaton::{FreeLHSAutomaton, FreeRHSAutomaton};
pub use dag_node::{FreeDagNode, RcFreeDagNode};
pub use free_net::{FreeNet, PatternSet, RcFreeNet};
pub use remainder::{FreeRemainder, FreeRemainderList, RcFreeRemainder};
pub use symbol::{FreeSymbol, RcFreeSymbol};
pub use term::{FreeTerm, RcFreeTerm};

use super::{LHSAutomaton, RcLHSAutomaton, RcTerm, Term};
use crate::{core::sort::RcSort, theory::variable::RcVariableTerm};


pub type FreeOccurrences = Vec<FreeOccurrence>;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FreeOccurrence {
  position:  i32,
  arg_index: i32,
  term:      *mut dyn Term,
}

impl FreeOccurrence {
  pub fn new(position: i32, arg_index: i32, term: *mut dyn Term) -> Self {
    FreeOccurrence {
      position,
      arg_index,
      term,
    }
  }

  pub fn dereference_term<T: 'static>(&self) -> &mut T {
    let term: &mut dyn Term = unsafe { &mut *self.term };

    if let Some(term) = term.as_any_mut().downcast_mut::<T>() {
      term
    } else {
      unreachable!("Could not dereference as the requested type of Term. This is a bug.")
    }
  }

  pub fn try_dereference_term<T: 'static>(&self) -> Option<&mut T> {
    let term: &mut dyn Term = unsafe { &mut *self.term };
    term.as_any_mut().downcast_mut::<T>()
  }

  pub fn term(&self) -> &mut dyn Term {
    unsafe { &mut *self.term }
  }
}

// These structs are specific to the free theory. The ACU theory has its own version.
#[derive(Clone, Eq, PartialEq)]
pub struct FreeVariable {
  position:  i32,
  arg_index: i32,
  var_index: i32,
  sort:      RcSort,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BoundVariable {
  position:  i32,
  arg_index: i32,
  var_index: i32,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct GroundAlien {
  position:  i32,
  arg_index: i32,
  alien:     *mut dyn Term,
}


impl GroundAlien {
  pub fn dereference_term<T: 'static>(&self) -> &mut T {
    let term: &mut dyn Term = unsafe { &mut *self.alien };

    if let Some(term) = term.as_any_mut().downcast_mut::<T>() {
      term
    } else {
      unreachable!("Could not dereference as FreeTerm. This is a bug.")
    }
  }
}


#[derive(Clone, PartialEq)]
pub struct NonGroundAlien {
  position:  i32,
  arg_index: i32,
  automaton: RcLHSAutomaton, //RefCell<dyn LHSAutomaton>,
}
