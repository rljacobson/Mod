/*!

A `Module` serves as a kind of symbol table and holds information local to a module.

*/

use std::default;

use super::{
  RcSort,
  RcSortConstraint,
  SortSet
};

use crate::{
  abstractions::{
    WeakCell,
    IString
  },
  theory::RcSymbol
};

// We need this trait so that we can have `ModuleItem` trait objects.
pub trait ModuleItem {
  /// From `ModuleTable::ModuleItem`. Gives `self.index_within_parent`.
  fn get_index_within_module(&self) -> u32;
  fn set_module_information(&mut self, module: WeakModule, index_within_module: u32);
  fn get_module(&self) -> WeakModule;
  // fn get_mut_module(&mut self) -> &mut Module;
}

/*
macro_rules! impl_module_item {
  ($struct_name:ident) => {
    impl ModuleItem for $struct_name {
      #[inline(always)]
      fn get_index_within_module(&self) -> u32 {
        self.index_within_parent
      }

      #[inline(always)]
      fn get_module(&self) -> WeakModule {
        self.parent_module.clone()
      }

      #[inline(always)]
      fn set_module_information(&mut self, module: WeakModule, index_within_module: u32) {
        self.parent_module = module;
        self.index_within_parent = index_within_module;
      }
    }
  };
}

pub(crate) use impl_module_item;
*/

#[derive(Copy, Clone, Default)]
pub enum ModuleStatus {
  #[default]
  Open,
  SortSetClosed,
  SignatureClosed,
  FixUpsClosed,
  TheoryClosed,
  StackMachineCompiled,
}

pub type WeakModule = WeakCell<Module>;
// Local alias for convenience.
type Status = ModuleStatus;

#[derive(Default)]
pub struct Module {
  // environment: RcEnvironment ,  // pointer to some object in which module exists
  pub status: ModuleStatus,

  // TODO: Does a module own its `Sorts`?
  pub sorts: SortSet,
  // connectedComponents: Vec<RcConnectedComponent> ,
  pub symbols: Vec<RcSymbol>,
  pub sort_constraints: Vec<RcSortConstraint>,
  // equations: Vec<RcEquation> ,
  // rules: Vec<RcRule> ,
  // strategies: Vec<RcRewriteStrategy> ,
  // strategyDefinitions: Vec<RcStrategyDefinition> ,
  // sortBdds: RcSortBdds ,
  // minimumSubstitutionSize: i32 ,
  // memoMap: RcMemoMap ,  // global memo map for all symbols in module

  // NamedEntity members
  /// An ID, a name given by the user.
  pub name: IString,
}

impl Module {

  // The `name` parameter is an ID, a name given by the user. It is called `id` in Maude. Here we use an interned
  // string.

  #[inline(always)]
  pub fn new(name: IString) -> Module {
    Module {
      name,
      ..Module::default()
    }
  }
}
