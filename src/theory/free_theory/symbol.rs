/*!

A symbol belonging to the free theory.

*/

use std::any::Any;
use std::rc::Rc;

use crate::{
  core::{
    Module,
    ModuleItem,
    SortConstraintTable,
    WeakModule
  },
  theory::{Symbol, symbol::SymbolMembers},
};

use super::{FreeNet, RcFreeNet};

pub type RcFreeSymbol = Rc<FreeSymbol>;

pub struct FreeSymbol {
  descrimination_net: RcFreeNet,

  // `SymbolMembers`
  symbol_members: SymbolMembers,

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

