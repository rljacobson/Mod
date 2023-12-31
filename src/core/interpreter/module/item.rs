/*!

The `ModuleItem` trait, implemented by things that live in modules. Implemented by `PreEquation` and `Symbol`.
`ModuleItem`s have weak references back to their `Module`s.

*/

use crate::core::module::WeakModule;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ModuleItemType {
  MembershipAxiom = 0x10000000,
  Equation        = 0x20000000,
  Rule            = 0x30000000,
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
