use std::any::Any;
use std::rc::Rc;
use crate::core::{OpDeclaration, RcSort};
use crate::theory::Symbol;
use crate::theory::symbol::SymbolMembers;

pub type RcVariableSymbol = Rc<VariableSymbol>;

pub struct VariableSymbol {
  // `SymbolMembers`
  symbol_members: SymbolMembers,

}

impl VariableSymbol {
  pub fn sort(&self) -> RcSort {
    // Maude: Temporary hack until sorts mechanism revised.
    let s = self.symbol_members.sort_table.get_op_declarations();
    assert_eq!(s.len(), 1usize, "s.length() != 1");
    let v: &OpDeclaration = s.first().unwrap();
    assert_eq!(v.len(), 1usize, "v.length() != 1");

    v.first().unwrap().clone()
  }
}

impl Symbol for VariableSymbol {

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

