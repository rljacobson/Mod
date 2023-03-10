/*!


*/

use std::{
  cmp::Ordering,
  rc::Rc
};

use dyn_clone::clone_trait_object;

use crate::{
  theory::{
    DagNode,
    Symbol
  },
  Substitution,
  OrderingValue,
};

// Todo: Use `reffers::rc1::Strong` instead of `Rc`.
pub type RcTerm = Rc<dyn Term>;


#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Flags {
  //	A subterm is stable if its top symbol cannot change under instantiation.
  Stable = 1,

  //	A subterm is in an eager context if the path to its root contains only
  //	eagerly evaluated positions.
  EagerContext = 2,

  //	A subterm "honors ground out match" if its matching algorithm guarantees
  //	never to to return a matching subproblem when all the terms variables
  //	are already bound.
  HonorsGroundOutMatch = 4
}


pub trait Term {
  /// Gives the top symbol of this term.
  fn symbol(&self) -> &dyn Symbol;

  /// Is the term stable?
  fn is_stable(&self) -> bool;

  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering;

  fn compare_dag_node(&self, other: &dyn DagNode) -> Ordering {
    if self.symbol() == other.symbol() {
      self.compare_dag_arguments(other)
    } else {
      self.symbol().compare(other.symbol())
    }
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering;
  
  fn partial_compare(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> OrderingValue {
    if !self.is_stable() {
      // Only used for `VariableTerm`
      return self.partial_compare_unstable(partial_substitution, other);
    }

    if self.symbol()  == other.symbol() {
      // Only used for `FreeTerm`
      return self.partial_compare_arguments(partial_substitution, other);
    }

    // Todo: Where is Equal case?
    if self.symbol().compare(other.symbol())  == Ordering::Less {
      OrderingValue::Less
    } else {
      OrderingValue::Greater
    }
  }


  /// Overridden in `VariableTerm`
  fn partial_compare_unstable(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> OrderingValue {
    OrderingValue::Unknown
  }

  /// Overridden in `FreeTerm`
  fn partial_compare_arguments(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> OrderingValue {
    OrderingValue::Unknown
  }

}

clone_trait_object!(Term);

/*
Implementers:
  ACUTerm
*/
