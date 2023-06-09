/*!

A `TermBag` is a cache of terms occurring in patterns or right hand sides that can be reused in building right hand
sides to enable common subexpression sharing both within rhs and lhs->rhs.

`TermBag` is a thin wrapper around `TermHashSet`.

*/

use std::{
  collections::HashSet,
  borrow::Borrow,
  hash::{Hash, Hasher}
};

use crate::{
  theory::{MaybeTerm, RcTerm, Term},
  abstractions::{FastHasher, TermHashSet}
};

#[derive(Default)]
pub struct TermBag {
  terms_usable_in_eager_context: TermHashSet,
  terms_usable_in_lazy_context: TermHashSet,
}

impl TermBag {

  #[inline(always)]
  pub fn new() -> Self {
    Self::default()
  }

  /// Inserts the matched term if it is not already present in the `TermBag`. If it is already in the `TermBag`, no
  /// action is taken. In debug mode, attempting to insert an existing term results in an error.
  #[inline(always)]
  pub(crate) fn insert_matched_term(&mut self, term: RcTerm, eager_context: bool) {
    // New matched terms can never replace built terms (which are available at zero cost) nor existing matched terms
    // (for which the cost of storing the extra pointer may already have been paid).
    let b = self.terms_usable_in_lazy_context.insert_no_replace(term.clone());
    assert!(!b, "TermBag should not insert a term that is already in the bag");
    if eager_context {
      let b = self.terms_usable_in_eager_context.insert_no_replace(term.clone());
      assert!(!b, "TermBag should not insert a term that is already in the bag");
    }
  }

  /// Inserts a built term, replacing any existing term within the `TermBag`. New built terms should not arise if there
  /// is an existing usable term in the appropriate context, so a warning is emitted.
  #[inline(always)]
  pub(crate) fn insert_built_term(&mut self, term: RcTerm, eager_context: bool) {
    // New built terms should not arise if there is an existing usable term in the appropriate context.
    if eager_context {
      let replaced = self.terms_usable_in_eager_context.insert_replace(term.clone());
      debug_assert!(replaced.is_none(), "re-insertion of {}", term.borrow());
    } else {
      let replaced = self.terms_usable_in_lazy_context.insert_replace(term.clone());
      debug_assert!(replaced.is_none(), "re-insertion of {}", term.borrow());
    }
  }

  #[inline(always)]
  pub fn contains<Q>(&self, term: &Q, eager_context: bool) -> bool
    where dyn Term: Borrow<Q>,
          Q: Hash + Eq + ?Sized
  {
    if eager_context {
      self.terms_usable_in_eager_context.contains(term)
    } else {
      self.terms_usable_in_lazy_context.contains(term)
    }
  }

  /// Finds the provided term in the term bag, returning `None` if it is not present.
  #[inline(always)]
  pub fn find<Q>(&self, term: &Q, eager_context: bool) -> MaybeTerm
    where dyn Term: Borrow<Q>,
                 Q: Hash + Eq + ?Sized
  {
    if eager_context {
      self.terms_usable_in_eager_context.find(term)
    } else {
      self.terms_usable_in_lazy_context.find(term)
    }
  }

  /// Fetches  the value from the set, returning `None` if it is not present.
  #[inline(always)]
  pub fn find_for_hash(&self,  hash: u64, eager_context: bool) -> MaybeTerm {
    if eager_context {
      self.terms_usable_in_eager_context.find_for_hash(hash)
    } else {
      self.terms_usable_in_lazy_context.find_for_hash(hash)
    }
  }
}
