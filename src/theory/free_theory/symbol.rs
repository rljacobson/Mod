/*!

A symbol belonging to the free theory.

*/

use std::rc::Rc;

use reffers::rc1::Weak;

use crate::{
    theory::Symbol,
    core::{
        ModuleItem,
        Module,
        RcModule,
        SortConstraintTable
    }
};

use super::{FreeNet, RcFreeNet};

pub type RcFreeSymbol = Rc<FreeSymbol>;

#[derive(Clone)]
pub struct FreeSymbol {
  descrimination_net: RcFreeNet,


  sort_constraint_table : SortConstraintTable,

  pub hash_value        : u32, // Unique integer for comparing symbols, also called order
  pub unique_sort_index : i32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
  pub match_index       : u32, // For fast matching
  pub arity             : u32,
  pub memo_flag         : u32,

  index_within_parent: u32,
  parent_module      : RcModule,
}


impl Symbol for FreeSymbol {
    fn get_hash_value(&self) -> u32 {
        self.hash_value
    }

    fn get_sort_constraint_table(&self) -> &SortConstraintTable {
        &self.sort_constraint_table
    }
}

impl ModuleItem for FreeSymbol {
    fn get_index_within_module(&self) -> u32 {
        self.index_within_parent
    }

    fn get_module(&self) -> RcModule {
        self.parent_module.clone()
    }

    fn set_module_information(&mut self, module: Weak<Module>, index_within_module: u32) {
        self.parent_module = module;
        self.index_within_parent = index_within_module;
    }
}
