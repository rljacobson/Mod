/*!

This hash set is special in that the fact that it is actually a hash map between an item's hash and the item itself is
kept explicit. Consequently, you can look up an item using _any_ value that hashes to the same thing as the item. The
`get` method has signature `get<T>(&self, key: T) -> Option<RcTerm> where T: Hash`. (Alternatively, use `get_ref`,
`remove`, etc.)

The primary use case is for structural sharing of terms in which an `RcTerm` is stored and can be cloned when the
structure is queried with a &Term.

The `HashSet` is really just a thin wrapper around a `HashMap`.

 */


use std::{
  collections::HashMap,
  borrow::Borrow,
  hash::{BuildHasher, Hash, Hasher}
};

use crate::{
  abstractions::FastHasherBuilder,
  theory::{RcTerm, Term, MaybeTerm}
};

#[derive(Clone, Default)]
pub struct TermHashSet {
  inner: HashMap<u64, RcTerm, FastHasherBuilder>,
}

impl TermHashSet {

  #[inline(always)]
  pub fn new() -> Self {
    Self {
      inner: HashMap::default()
    }
  }

  #[inline(always)]
  pub fn contains<Q>(&self, value: &Q) -> bool
      where dyn Term: Borrow<Q>,
                   Q: Hash + Eq + ?Sized
  {
    // TODO: What is the best way to do this? Use `self.innner.hasher()`? Call `value.hash(..)` or
    //       `value.compute_hash()`? Also, should `compute_hash()` return a `u32` or `u64`?
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();
    self.inner.contains_key(&key)
  }

  /// Inserts the value into the set, returning true if the value was not already present.
  #[inline(always)]
  pub fn insert_replace(&mut self, value: RcTerm) -> MaybeTerm {
    // TODO: Same questions as in `contains`.
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();
    self.inner.insert(key, value)
  }

  /// Inserts the value into the set if it is not already present, returning true if the value was not already present.
  #[inline(always)]
  pub fn insert_no_replace(&mut self, value: RcTerm) -> bool {
    // TODO: Same questions as in `contains`.
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();

    if self.inner.contains_key(&key) {
      return false;
    }
    self.inner.insert(key, value);
    return true;
  }

  /// Fetches the value from the set, returning `None` if it is not present.
  #[inline(always)]
  pub fn find_for_hash(&self,  hash: u64) -> MaybeTerm {
    self.inner.get(&hash).cloned()
  }

  /// Finds the provided (borrowed) term, if it is in the set.
  #[inline(always)]
  pub fn find<Q>(&self, value: &Q) -> MaybeTerm
    where dyn Term: Borrow<Q>,
                 Q: Hash + Eq + ?Sized
  {
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();
    self.find_for_hash(key)
  }
}
