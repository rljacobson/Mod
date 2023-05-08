/*!

The Symbol trait and its concrete implementations BinarySymbol and AssociativeSymbol.

A Symbol implements the traits:
RuleTable,
NamedEntity,
LineNumber,
SortTable,
SortConstraintTable,
EquationTable,
Strategy,
MemoTable

*/

use std::{
  cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
  rc::Rc,
};
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};

use crate::{
  abstractions::{
    Set,
    IString
  },
  core::{
    ModuleItem,
    RcSort,
    Sort,
    SortConstraintTable,
    SortTable,
    WeakModule
  },
  theory::{
    RcDagNode,
    RcTerm
  },
};


pub type RcSymbol = Rc<dyn Symbol>;
pub type SymbolSet = Set<dyn Symbol>;

/*
One way to deal with a lack of trait data members is to have a struct containing the shared members and then
either
  1. have a macro that implements the getters and setters, or
  2. have a trait-level getter for the struct that is implemented in every implementor, and have
     shared-implementation at the trait level by using the getter in the `impl Trait`.
We choose the second option.
*/
#[derive(PartialEq, Eq)]
pub struct SymbolMembers {
  /// `NamedEntity` members
  pub name: IString,

  /// `Symbol` members
  pub hash_value        : u32, // Unique integer for comparing symbols, also called order
  pub unique_sort_index : i32, // Slow Case: 0, Fast Case: -1, positive for symbols that only produce an unique sort
  pub match_index       : u32,       // For fast matching
  pub arity             : u32,
  pub memo_flag         : bool,

  /// `SortConstraintTable` members.
  /// It is Maude's Symbol superclass, but we use composition instead.
  pub sort_constraint_table: SortConstraintTable,

  /// `SortTable` is Maude's Symbol superclass, but we use composition instead.
  pub sort_table: SortTable,

  // `ModuleItem`
  pub(crate) index_within_parent : u32,
  pub(crate) parent_module       : WeakModule,
}

impl SymbolMembers {
  pub fn new(name: IString, arity: u32, memo_flag: bool) -> SymbolMembers {
    let mut new_symbol =
      SymbolMembers{
        name,
        hash_value: 0,
        unique_sort_index: 0,
        match_index: 0,
        arity,
        memo_flag,
        sort_constraint_table: Default::default(),
        sort_table: Default::default(),
        index_within_parent: 0,
        parent_module: Default::default(),
      };
    // The only time the hash is computed.
    new_symbol.hash_value = new_symbol.compute_hash();

    new_symbol
  }

  fn compute_hash(&self) -> u32 {
    // In Maude, the hash value is the number (chronological order of creation) of the symbol OR'ed
    // with (arity << 24). Here we swap the "number" with the hash of the IString as defined by the
    // IString implementation.
    // ToDo: This… isn't great, because the hash is 32 bits, not 24, and isn't generated in numeric order.
    IString::get_hash(&self.name) | (self.arity << 24)
  }
}

pub trait Symbol {

  // region Member Getters and Setters
  /// Trait level access to members for shared implementation
  fn symbol_members(&self) -> &SymbolMembers;
  fn symbol_members_mut(&mut self) -> &mut SymbolMembers;

  #[inline(always)]
  fn name(&self) -> IString {
    self.symbol_members().name.clone()
  }

  /// Same as `get_order`
  #[inline(always)]
  fn get_hash_value(&self) -> u32 {
    self.symbol_members().hash_value
  }

  #[inline(always)]
  fn arity(&self) -> u32 {
    self.symbol_members().arity
  }

  #[inline(always)]
  fn sort_constraint_table(&self) -> &SortConstraintTable {
    &self.symbol_members().sort_constraint_table
  }

  #[inline(always)]
  fn sort_constraint_table_mut(&mut self) -> &mut SortConstraintTable {
    &mut self.symbol_members_mut().sort_constraint_table
  }
  // endregion

  // Note: `compute_base_sort` is a method of *Symbol in Maude.
  // However, it takes its owning DagNode as a parameter, subject.
  // fn compute_base_sort(&self, subject: &mut dyn DagNode);
  #[inline(always)]
  fn sort_constraint_free(&self) -> bool {
    self.sort_constraint_table().is_empty()
  }

  #[inline(always)]
  fn sort_table(&self) -> &SortTable {
    &self.symbol_members().sort_table
  }

  #[inline(always)]
  fn compare(&self, other: &dyn Symbol) -> Ordering {
    // This is just std::Ord::cmp(self, other)
    // Ord::cmp(&self, other)
    self.get_hash_value().cmp(&other.get_hash_value())
  }

  fn as_any(&self) -> &dyn Any;

}

//  region Order and Equality impls
impl PartialOrd for dyn Symbol {
  #[inline(always)]
  fn partial_cmp(&self, other: &dyn Symbol) -> Option<Ordering> {
    let result = self.get_hash_value().cmp(&other.get_hash_value());
    Some(result)
  }
}

impl Ord for dyn Symbol {
  #[inline(always)]
  fn cmp(&self, other: &dyn Symbol) -> Ordering {
    self.get_hash_value().cmp(&other.get_hash_value())
  }
}

impl Eq for dyn Symbol {}

impl PartialEq for dyn Symbol {
  #[inline(always)]
  fn eq(&self, other: &dyn Symbol) -> bool {
    self.get_hash_value() == other.get_hash_value()
  }
}
// endregion


// Every `Symbol` is a `ModuleItem`
impl ModuleItem for dyn Symbol {
#[inline(always)]
fn get_index_within_module(&self) -> u32 {
  self.symbol_members().index_within_parent
}

  #[inline(always)]
  fn set_module_information(&mut self, module: WeakModule, index_within_module: u32) {
    self.symbol_members_mut().parent_module       = module;
    self.symbol_members_mut().index_within_parent = index_within_module;
  }

  #[inline(always)]
  fn get_module(&self) -> WeakModule {
    self.symbol_members().parent_module.clone()
  }
}


impl Display for dyn Symbol {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.symbol_members().name)
  }
}

impl Debug for dyn Symbol {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.symbol_members().name)
  }
}


/*
Deriving Traits:
BinarySymbol
*/

pub trait BinarySymbol: Symbol {
  fn get_identity(&self) -> Option<RcTerm>;
  fn get_identity_dag(&self) -> Option<RcDagNode>;
}

