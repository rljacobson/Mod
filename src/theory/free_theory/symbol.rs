/*!

A symbol belonging to the free theory.

*/

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::{
  abstractions::{IString, RcCell},
  core::Strategy,
  theory::{
    DagNode,
    NodeList,
    RcDagNode,
    RcTerm,
    Symbol,
    SymbolMembers,
  },
};

use super::{FreeNet, RcFreeNet, FreeDagNode, FreeTerm};

pub type RcFreeSymbol = Rc<FreeSymbol>;

pub struct FreeSymbol {
  discrimination_net: RcFreeNet,

  // `SymbolMembers`
  symbol_members: SymbolMembers,

  // `Strategy`
  strategy: Strategy
}

impl FreeSymbol {
  pub fn new(name: IString, arity: u32, memo_flag: bool, strategy: Strategy) -> FreeSymbol {
    FreeSymbol{
      discrimination_net: Default::default(),
      symbol_members: SymbolMembers::new(name, arity, memo_flag),
      strategy
    }
  }

  pub fn make_term_with_args(self, args: Vec<RcTerm>) -> FreeTerm {
    FreeTerm::with_args(Rc::new(self), args)
  }
}

impl Symbol for FreeSymbol {

  #[inline(always)]
  fn symbol_members(&self) -> &SymbolMembers {
    &self.symbol_members
  }

  #[inline(always)]
  fn symbol_members_mut (&mut self) -> &mut SymbolMembers{
    &mut self.symbol_members
  }

  #[inline(always)]
  fn as_any(&self) -> &dyn Any {
    self
  }

}

