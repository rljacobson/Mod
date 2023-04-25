/*!


*/

use std::{
  cmp::Ordering,
  rc::Rc,
  any::Any
};

use crate::{
  core::{
    Substitution,
    OrderingValue,
    RcConnectedComponent,
    NatSet
  },
  abstractions::{RcCell, Set},
  theory::{
    RcSymbol,
    DagNode,
    NodeList,
    Symbol,
    symbol::SymbolSet
  }
};


pub type RcTerm = RcCell<dyn Term>;
pub type TermSet = Set<dyn Term>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TermKind {
  Free,
  Bound,
  Ground,
  NonGround
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(crate) enum TermFlags {
  ///	A subterm is stable if its top symbol cannot change under instantiation.
  Stable = 1,

  ///	A subterm is in an eager context if the path to its root contains only
  ///	eagerly evaluated positions.
  EagerContext = 2,

  ///	A subterm "honors ground out match" if its matching algorithm guarantees
  ///	never to to return a matching subproblem when all the terms variables
  ///	are already bound.
  HonorsGroundOutMatch = 4
}

/*
One way to deal with a lack of trait data members is to have a struct containing the shared members and then
either
  1. have a macro that implements the getters and setters, or
  2. have a trait-level getter for the struct that is implemented in every implementor, and have
     shared-implementation at the trait level by using the getter in the `impl Trait`.
We choose the second option.
*/
pub struct TermMembers {
  pub(crate) top_symbol         : RcSymbol,
  pub(crate) occurs_set         : NatSet,
  pub(crate) context_set        : NatSet,
  pub(crate) collapse_set       : SymbolSet,
  pub(crate) flags              : u8,
  pub(crate) sort_index         : i32, //i16,
  pub(crate) connected_component: RcConnectedComponent,
  pub(crate) save_index         : i32,
  pub(crate) hash_value         : u32,
  pub(crate) cached_size        : i32,

  // Static Members
  // ToDo : Figure out what to do with `Term`'s static members.
  pub(crate) sub_dags          : NodeList,
  pub(crate) converted         : TermSet,
  pub(crate) set_sort_info_flag: bool
}



pub trait Term {
  /// Gives the top symbol of this term.
  fn symbol(&self) -> RcSymbol {
    self.term_members().top_symbol.clone()
  }

  /// Access to data members. This allows shared implementation in the trait implementation rather than generic
  /// implementation being reproduced for every implementor of the trait.
  fn term_members(&self) -> &TermMembers;
  fn term_members_mut(&mut self) -> &mut TermMembers;


  /// Is the term stable?
  fn is_stable(&self) -> bool {
    self.term_members().flags & TermFlags::Stable as u8 != 0
  }

  /// Downcasts to Self
  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering;

  fn compare_dag_node(&self, other: &dyn DagNode) -> Ordering {
    if self.symbol().get_hash_value() == other.symbol().get_hash_value() {
      self.compare_dag_arguments(other)
    } else {
      self.symbol().compare(other.symbol().as_ref())
    }
  }

  /// Downcasts to Self
  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering;


  fn partial_compare(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> OrderingValue {
    if !self.is_stable() {
      // Only used for `VariableTerm`
      return self.partial_compare_unstable(partial_substitution, other);
    }

    if self.symbol().get_hash_value() == other.symbol().get_hash_value() {
      // Only used for `FreeTerm`
      return self.partial_compare_arguments(partial_substitution, other);
    }


    if self.symbol().compare(other.symbol().as_ref())  == Ordering::Less {
      OrderingValue::Less
    } else {
      OrderingValue::Greater
    }
  }


  fn compare(&self, other: &dyn Term) -> Ordering {
    let other_symbol = other.symbol();
    let r = self.symbol().compare(other_symbol.as_ref());
    if r == Ordering::Equal {
      return self.compare_term_arguments(other);
    }
    return r;
  }

  fn as_any(&self) -> &dyn Any;

  /// Overridden in `VariableTerm`
  fn partial_compare_unstable(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> OrderingValue {
    OrderingValue::Unknown
  }

  /// Overridden in `FreeTerm`
  fn partial_compare_arguments(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> OrderingValue {
    OrderingValue::Unknown
  }

}

/*
Implementers:
  ACUTerm
  FreeTerm
*/
