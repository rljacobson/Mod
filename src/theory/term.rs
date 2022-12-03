/*!


*/

use dyn_clone::{clone_trait_object, DynClone};

use crate::Substitution;
use crate::theory::DagNode;
use crate::theory::Symbol;

// In the Maude source, this enum is called `ReturnValue`. We don't use `std::cmp::Ordering` because we need
// Unknown/Undecided.
// Todo: Instead of this custom enum, should we have `Option<Ordering>` or `Result<Ordering, ()>`?
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum OrderingValue {
  Greater = 1,
  Less = -2,
  Equal = 0,
  Unknown = -1
}

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


pub trait Term: DynClone {
  /// Gives the top symbol of this term.
  fn symbol(&self) -> &Symbol;

  /// Is the term stable?
  fn is_stable(&self) -> bool;

  fn compare_term_arguments(&self, other: &dyn Term) -> u32;

  fn compare_dag_node(&self, other: &dyn DagNode) -> u32 {
    if self.symbol() == other.symbol() {
      self.compare_dag_arguments(other)
    } else {
      self.symbol().compare(other.symbol())
    }
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> u32;

  fn partial_compare(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> OrderingValue {
    if !self.stable() {
      // Only used for `VariableTerm`
      return self.partial_compare_unstable(partial_substitution, other);
    }

    if self.symbol()  == other.symbol() {
      // Only used for `FreeTerm`
      return self.partial_compare_arguments(partial_substitution, other);
    }

    if self.symbol().compare(other.symbol())  <  0 {
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
