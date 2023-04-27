/*!


*/

use std::{
  cmp::Ordering,
  rc::Rc,
  any::Any
};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

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
use crate::core::SpecialSort;
use crate::theory::{DagNodeFlag, DagNodeFlags, RcDagNode};


pub type RcTerm = RcCell<dyn Term>;
pub type TermSet = Set<dyn Term>;
pub type NodeCache = HashMap<*const dyn Term, RcDagNode>;

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

  // This is the HashMap of dag nodes that allows structural sharing. Maude implements it with two structures. It is
  // reset on each call to term2dag and is only used during dagification. It should be able to be replaced with a
  // parameter to `dagify()` in all cases.
  // Note: `dagify2()` is the theory specific part of `dagify()`.
  // pub(crate) sub_dags          : NodeList,
  // pub(crate) converted         : TermSet,

  // This is only used twice:
  //   1. CachedDag::getDag()
  //   2. SubtermTask::SubtermTask
  // It should be able to be replaced with a parameter to `dagify()` in all cases.
  // pub(crate) set_sort_info_flag: bool
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

  /// Create a directed acyclic graph from this term.
  fn dagify(&self, sub_dags: &mut NodeCache, set_sort_info: bool) -> RcDagNode {
    match sub_dags.entry(self) {

      Entry::Occupied(entry) => {
        entry.clone()
      }

      Entry::Vacant(entry) => {
        let mut d = self.dagify_aux();
        if set_sort_info {
          assert_ne!(self.sort_index, SpecialSort::Unknown, "Missing sort info");
          let mut d = d.borrow_mut();
          d.set_sort_index(self.sort_index);
          d.set_flags(DagNodeFlag::Reduced.into());
        }
        entry.insert(d.clone());
        d
      }

    }
  }

  /// Create a directed acyclic graph from this term. This method has the implementation-specific stuff.
  fn dagify_aux(&self, sub_dags: &mut NodeCache, set_sort_info: bool) -> RcDagNode;


  #[inline(always)]
  fn as_ptr(&self) -> *const dyn Term {
    self as *const dyn Term
  }

}

/*
Implementers:
  ACUTerm
  FreeTerm
*/
