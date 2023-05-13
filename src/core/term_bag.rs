/*!

A `TermBag` is a cache of terms occurring in patterns or right hand sides that can be reused in building right hand
sides to enable common subexpression sharing both within rhs and lhs->rhs.

*/

use std::collections::HashSet;
use crate::theory::{RcTerm, Term};

pub type TermHashSet = HashSet<RcTerm>;

pub struct TermBag {
  terms_usable_in_eager_context: TermHashSet,
  terms_usable_in_lazy_context: TermHashSet,
}

impl TermBag {
  pub(crate) fn insert_matched_term(&mut self, term: RcTerm, eager_context: bool) {

    // New matched terms can never replace built terms (which are available at zero cost) nor existing matched terms
    // (for which the cost of storing the extra pointer may already have been paid).
    self.terms_usable_in_lazy_context.insert(term.clone());
    if eager_context {
      self.terms_usable_in_eager_context.insert(term.clone());
    }
  }

  fn insert_built_term(&mut self, term: RcTerm, eager_context: bool) {
    // New built terms should not arise if there is an existing usable term in the appropriate context.
    if eager_context {
      let inserted = self.terms_usable_in_eager_context.insert(term.clone());
      debug_assert!(!inserted, "re-insertion of {}", term.borrow());
    } else {
      let inserted = self.terms_usable_in_lazy_context.insert(term.clone());
      debug_assert!(inserted, "re-insertion of {}", term.borrow());
    }
  }

  fn find_term(&self, term: RcTerm, eager_context: bool) -> Option<RcTerm> {
    if eager_context {
      self.terms_usable_in_eager_context.get(&term).cloned()
    } else {
      self.terms_usable_in_lazy_context.get(&term).cloned()
    }
  }
}
