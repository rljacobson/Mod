/*!


*/



use crate::Substitution;
use crate::theory::dag_node::DagNode;
use crate::theory::symbol::Symbol;

// todo: These are poorly named, but this is their name in the Maude source.
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum ReturnValue {
  Greater = 1,
  Less = -2,
  Equal = 0,
  Unknown = -1
}

#[derive(PartialEq, Eq)]
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
  fn symbol(&self) -> &Symbol;

  /// Is the term stable?
  fn is_stable(&self) -> bool;

  fn compare_term_arguments(&self, other: &dyn Term) -> u32;

  fn compare_dag_node(&self, other: &dyn DagNode) -> u32 {
    let value = self.symbol().compare(other.symbol());
    if value != 0 {
      self.compare_dag_arguments(other)
    } else {
      value
    }
  }

  fn compare_dag_arguments(&self, other: &dyn DagNode) -> u32;

  fn partial_compare(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> ReturnValue {
    if !self.stable() {
      // Only used for `VariableTerm`
      return self.partial_compare_unstable(partial_substitution, other);
    }

    if self.symbol()  == other.symbol() {
      // Only used for `FreeTerm`
      return self.partial_compare_arguments(partial_substitution, other);
    }

    if self.symbol().compare(other.symbol())  <  0 {
      ReturnValue::Less
    } else {
      ReturnValue::Greater
    }
  }


  /// Overridden in `VariableTerm`
  fn partial_compare_unstable(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> ReturnValue {
    ReturnValue::Unknown
  }

  /// Overridden in `FreeTerm`
  fn partial_compare_arguments(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> ReturnValue {
    ReturnValue::Unknown
  }

}

/*
Implementers:
  ACUTerm
*/
