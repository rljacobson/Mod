/*!

This hash set is special in that the fact that it is actually a hash map between an item's hash and the item itself is
kept explicit. Consequently, you can look up an item using _any_ value that hashes to the same thing as the item. The
`get` method has signature `get<T>(&self, key: T) -> Option<RcTerm> where T: SemanticHash`. (Alternatively, use `get_ref`,
`remove`, etc.)

The primary use case is for structural sharing of terms in which an `RcTerm` is stored and can be cloned when the
structure is queried with a &Term. Thus, contained items must implement `SemanticHash` rather than `Hash`.

The `HashSet` is really just a thin wrapper around a `HashMap` (but using `SemanticHash` instead of `Hash`).

 */


use std::{
  collections::HashMap,
  borrow::Borrow,
  hash::{
    BuildHasher,
    Hash,
    Hasher
  }
};
use std::collections::hash_map::Entry;

use crate::{
  abstractions::FastHasherBuilder,
  theory::{
    RcTerm,
    RcDagNode
  }
};
use crate::abstractions::RcCell;
use crate::core::hash_cons_set::HashConsSet;

pub type TermHashSet = HashSet<RcTerm>;
pub type DagNodeHashSet = HashConsSet;

pub type HashValueType = u64;


#[derive(Clone)]
pub struct HashSet<T> {
  inner: HashMap<HashValueType, T, FastHasherBuilder>,
}

impl<T> Default for HashSet<T>{
  fn default() -> Self {
    HashSet{
      inner: HashMap::<HashValueType, T, FastHasherBuilder>::new()
    }
  }
}

impl<T> HashSet<T>
    where T: Clone + Hash
{

  #[inline(always)]
  pub fn new() -> Self {
    Self {
      inner: HashMap::default()
    }
  }

  /// Inserts the value into the set, returning true if the value was not already present.
  pub fn insert_replace(&mut self, value: T) -> Option<T> {
    // TODO: Same questions as in `contains`.
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();
    self.inner.insert(key, value)
  }

  /// Inserts the value into the set if it is not already present, returning `(found_value, not_present)`, where
  /// `not_present` is true if the value was not already present.
  pub fn insert_no_replace(&mut self, value: T) -> (T, bool) {
    // TODO: Same questions as in `contains`.
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();

    match self.inner.entry(key) {
      Entry::Occupied(entry) => {
        (entry.get().clone(), false)
      }
      Entry::Vacant(entry) => {
        entry.insert(value.clone());
        (value, true)
      }
    }
  }

  /// Fetches the value from the set, returning `None` if it is not present.
  #[inline(always)]
  pub fn find_for_hash(&self, hash: HashValueType) -> Option<T> {
    self.inner.get(&hash).cloned()
  }
}

// In this impl, `T = Rc<U>`
impl<U: ?Sized> HashSet<RcCell<U>>
{
  pub fn contains<Q>(&self, value: &Q) -> bool
    where U: Borrow<Q>,
          Q: Hash + Eq + ?Sized
  {
    // TODO: What is the best way to do this? Use `self.inner.hasher()`? Call `value.hash(..)` or
    //       `value.compute_hash()`? Also, should `compute_hash()` return a `u32` or `HashValueType`?
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();
    self.inner.contains_key(&key)
  }

  /// Finds the provided (borrowed) term, if it is in the set.
  pub fn find<Q>(&self, value: &Q) -> Option<(RcCell<U>, HashValueType)>
    where U: Borrow<Q>,
                 Q: Hash + Eq + ?Sized
  {
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();
    self.find_for_hash(key).map(|v| (v, key))
  }
}
