/*!

A `TermBag` is a cache of terms occurring in patterns or right hand sides that can be reused in building right hand
sides to enable common subexpression sharing both within rhs and lhs->rhs.

A special __semantic hash__ is used to determine if two terms are "the same." Sameness means only that a term from the
term bag can be used in place of the given term. In fact, even when two terms are semantically the same, there may be
reasons why a separate copy needs to be used instead of a shared cached term. In particular, if local changes need to be
made to a term without effecting all other semantically identical terms, then the globally shared cached term can't be
used.

We need a bimap between semantic hash and terms. The simplest way is to use the semantic hash as a key. Then

    term -> semantic hash  : just the semantic hash function itself
    semantic hash -> term  : a lookup in the "term bag."

Maude uses "small" numbers in place of the semantic hash. The term bag must keep track of these numbers, and they are
not deterministic. It isn't clear to me why.

In Maude, a `TermBag` is a thin wrapper around a PointerSet.

*/

use std::{
  borrow::Borrow,
  hash::{Hash, Hasher},
};

use crate::{
  abstractions::{FastHasher, TermHashSet},
  theory::{MaybeTerm, RcTerm, Term},
};
use crate::abstractions::HashValueType;

pub struct TermBag {
  terms_usable_in_eager_context: TermHashSet,
  terms_usable_in_lazy_context:  TermHashSet,
}

impl Default for TermBag {
  fn default() -> Self {
    TermBag {
      terms_usable_in_eager_context: TermHashSet::new(),
      terms_usable_in_lazy_context:  TermHashSet::new(),
    }
  }
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
    let (_, b) = self.terms_usable_in_lazy_context.insert_no_replace(term.clone());
    assert!(!b, "TermBag should not insert a term that is already in the bag");
    if eager_context {
      let (_, b) = self.terms_usable_in_eager_context.insert_no_replace(term.clone());
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
  where
    dyn Term: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    if eager_context {
      self.terms_usable_in_eager_context.contains(term)
    } else {
      self.terms_usable_in_lazy_context.contains(term)
    }
  }

  /// Finds the provided term in the term bag, returning `None` if it is not present.
  #[inline(always)]
  pub fn find<Q>(&self, term: &Q, eager_context: bool) -> Option<(RcTerm, HashValueType)>
  where
    dyn Term: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    if eager_context {
      self.terms_usable_in_eager_context.find(term)
    } else {
      self.terms_usable_in_lazy_context.find(term)
    }
  }

  /// Fetches  the value from the set, returning `None` if it is not present.
  #[inline(always)]
  pub fn find_for_hash(&self, hash: u64, eager_context: bool) -> MaybeTerm {
    if eager_context {
      self.terms_usable_in_eager_context.find_for_hash(hash)
    } else {
      self.terms_usable_in_lazy_context.find_for_hash(hash)
    }
  }
}
