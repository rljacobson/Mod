/*!

Free theory automaton.

*/

use std::{ops::DerefMut, any::Any};

use crate::{
  theory::{
    automaton::{
      BoundVariable,
      FreeVariable,
      GroundAlien,
      LHSAutomaton,
      NonGroundAlien
    },
    DagNode,
    RcDagNode,
    RcSymbol,
    MaybeSubproblem,
    ExtensionInfo,
    Symbol,
  },
  core::Substitution
};

use super::super::{
  RcFreeSymbol,
  FreeSymbol,
  FreeDagNode,
  RcFreeDagNode,
};


#[derive(Clone)]
struct FreeSubterm {
  position  : u32,
  argIndex  : u32,
  symbol    : RcSymbol,
  saveIndex : u32,
}

pub struct FreeLHSAutomaton {
  top_symbol: RcFreeSymbol,

  // TODO: This is supposed to be a list of lists of RcDagNodes?
  stack               : Vec<RcDagNode>,
  free_subterms       : Vec<FreeSubterm>,
  uncertain_variables : Vec<FreeVariable>,
  bound_variables     : Vec<BoundVariable>,
  ground_aliens       : Vec<GroundAlien>,

  // ToDo: These are owned by `FreeLHSAutomaton`.
  nonGroundAliens: Vec<NonGroundAlien>,
}


impl LHSAutomaton for FreeLHSAutomaton {
  fn match_(
    &mut self,
    subject        : RcDagNode,
    solution       : &mut Substitution,
    extension_info : Option<&mut dyn ExtensionInfo>,
  ) -> (bool, MaybeSubproblem)
  {
    if subject.as_ref().symbol().as_ref() != self.top_symbol.as_ref() as &dyn Symbol {
      return (false, None);
    }

    if self.top_symbol.arity == 0 {
      return (true, None);
    }

    // Maude casts to a FreeDagNode?!
    if let Some(s) = subject.borrow_mut().as_any_mut().downcast_mut::<FreeDagNode>() {
    

    } else {
      panic!("ACULHSAutomaton::match  called with non ACU DagNode. This is a bug.");
    }


    (false, None)
  }
}
