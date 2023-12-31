use string_cache::DefaultAtom as IString;
use tiny_logger::{Channel::Debug, log};

use crate::{
  core::{
    module::{StatementProfile, SymbolProfile},
    pre_equation::RcPreEquation,
    sort::{RcConnectedComponent, SortSet},
  },
  theory::RcSymbol,
};

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

// Local alias for convenience.
type Status = ModuleStatus;

#[derive(Default)]
pub struct Module {
  // environment: RcEnvironment ,  // pointer to some object in which module exists
  pub status: ModuleStatus,

  // TODO: Does a module own its `Sorts`?
  pub sorts:               SortSet,
  pub connectedComponents: Vec<RcConnectedComponent>,
  pub symbols:             Vec<RcSymbol>,
  pub sort_constraints:    Vec<RcPreEquation>,
  pub equations:           Vec<RcPreEquation>,
  pub rules:               Vec<RcPreEquation>,

  // strategies: Vec<RcRewriteStrategy> ,
  // strategyDefinitions: Vec<RcStrategyDefinition> ,
  // sortBdds: RcSortBdds ,
  pub(crate) minimum_substitution_size: i32,

  // pub memo_map: BxMemoMap ,  // Memoization map for all symbols in module

  // NamedEntity members
  /// An ID, a name given by the user.
  pub name: IString,

  // ProfileModule members
  symbol_info: Vec<SymbolProfile>,
  mb_info    : Vec<StatementProfile>, // Membership
  eq_info    : Vec<StatementProfile>, // Equation
  rl_info    : Vec<StatementProfile>, // Rule
  // sd_info    : Vec<StatementProfile>, // Strategy Definition
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
      log(
        Debug,
        5,
        format!(
          "minimumSubstitutionSize for {:?} increased from {} to {}",
          self.name, self.minimum_substitution_size, minimum_size
        )
        .as_str(),
      );
      self.minimum_substitution_size = minimum_size;
    }
  }

}
