/*!

A `Module` serves as a kind of symbol table and holds information local to a module. It's the equivalent of a
translation unit in C++ or a module in Python or Rust.

`ModuleItem`s are objects that are numbered within a module. This provides us with:
  (1) a way of getting back to the module containing an object; and
  (2) a number that is useful for indexing.

*/

mod profile;

use pratt::{
  Channel::Debug,
  log
};

use crate::{
  abstractions::{
    WeakCell,
    IString
  },
  core::{
    sort::{
      RcSort,
      SortSet
    },
  },
  theory::RcSymbol,
};

pub use profile::{
  SymbolProfile,
  FragmentProfile,
  StatementProfile,
};
use crate::core::pre_equation::RcPreEquation;
use crate::core::sort::RcConnectedComponent;


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ItemType {
  MembershipAxiom    = 0x10000000,
  Equation           = 0x20000000,
  Rule               = 0x30000000,
  // StratDecl          = 0x40000000, // Unimplemented
  // StrategyDefinition = 0x50000000, // Unimplemented
}

// We need this trait so that we can have `ModuleItem` trait objects.
pub trait ModuleItem {
  /// From `ModuleTable::ModuleItem`. Gives `self.index_within_parent`.
  fn get_index_within_module(&self) -> i32;
  fn set_module_information(&mut self, module: WeakModule, index_within_module: i32);
  fn get_module(&self) -> WeakModule;
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
  pub sorts              : SortSet,
  pub connectedComponents: Vec<RcConnectedComponent>,
  pub symbols            : Vec<RcSymbol>,
  pub sort_constraints   : Vec<RcPreEquation>,
  pub equations          : Vec<RcPreEquation>,
  pub rules              : Vec<RcPreEquation>,
  // strategies: Vec<RcRewriteStrategy> ,
  // strategyDefinitions: Vec<RcStrategyDefinition> ,
  // sortBdds: RcSortBdds ,

  pub(crate) minimum_substitution_size: i32,

  // memoMap: RcMemoMap ,  // global memo map for all symbols in module

  // NamedEntity members
  /// An ID, a name given by the user.
  pub name: IString,


  // ProfileModule members
  symbol_info: Vec<SymbolProfile>,
  mb_info    : Vec<StatementProfile>, // Membership
  eq_info    : Vec<StatementProfile>, // Equation
  rl_info    : Vec<StatementProfile>, // Rule
  sd_info    : Vec<StatementProfile>, // Strategy Definition
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

  #[inline(always)]
  pub fn notify_substitution_size(&mut self, minimum_size: i32) {
    if minimum_size > self.minimum_substitution_size {

      log(Debug,
        5,
        format!(
          "minimumSubstitutionSize for {:?} increased from {} to {}",
          self.name,
          self.minimum_substitution_size,
          minimum_size
        ).as_str()
      );
      self.minimum_substitution_size = minimum_size;
    }
  }

}
