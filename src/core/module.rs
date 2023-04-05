use std::default;

use super::{RcSort, RcSortConstraint, SortSet};

use crate::{abstractions::WeakCell, theory::RcSymbol};

// We need this trait so that we can have `ModuleItem` trait objects.
pub trait ModuleItem {
    /// From `ModuleTable::ModuleItem`. Gives `self.index_within_parent`.
    fn get_index_within_module(&self) -> u32;
    fn set_module_information(&mut self, module: WeakkModule, index_within_module: u32);
    fn get_module(&self) -> WeakkModule;
    // fn get_mut_modeule(&mut self) -> &mut Module;
}

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

pub type WeakkModule = WeakCell<Module>;
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
    pub name: u32, // TODO: Probably replace with `InternedString`.
}

impl Module {
    // The `name` parameter is an ID, a name given by the user.
    // It is called `id` in Maude.
    pub fn new(name: u32) -> Module {
        Module {
            name,
            ..Module::default()
        }
    }
}
