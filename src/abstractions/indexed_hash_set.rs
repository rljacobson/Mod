/*!

Similar to a HashSet, except an index equal to original insertion order is assigned to each element,
and fast look-ups can be done using the index.

*/

use std::{
  borrow::Borrow,
  collections::{hash_map::Entry, HashMap},
  hash::{BuildHasher, Hash, Hasher},
  rc::Rc,
};

use crate::abstractions::FastHasherBuilder;

/// Similar to a HashSet, except an index equal to original insertion order is assigned to each
/// element, and fast look-ups can be done using the index.
#[derive(Clone)]
pub struct IndexedHashSet<T> {
  inner:  HashMap<u64, (T, usize), FastHasherBuilder>,
  hashes: Vec<u64>,
}

impl<T> Default for IndexedHashSet<T> {
  fn default() -> Self {
    IndexedHashSet {
      inner: Default::default(),
      hashes: vec![],
    }
  }
}

impl<T> IndexedHashSet<T>
where
  T: Clone + Hash,
{
  #[inline(always)]
  pub fn new() -> Self {
    Self::default()
  }

  /// Inserts the value into the set, returning the index of the value. If the value is already in
  /// the set, it will be replaced with the new value.
  pub fn insert_replace(&mut self, value: T) -> (T, usize) {
    // TODO: Same questions as in `contains`.
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();
    let mut entry = self.inner.entry(key);
    match entry {
      Entry::Vacant(vacant_entry) => {
        let index = self.hashes.len();

        self.hashes.push(key);
        vacant_entry.insert((value.clone(), index));
        (value, index)
      }

      Entry::Occupied(mut occupied_entry) => {
        let (found, idx): &mut (T, usize) = occupied_entry.get_mut();
        *found = value.clone();
        (value, *idx)
      }
    }
  }

  /// Inserts the value into the set if it is not already present, returning true if the value was not already present.
  pub fn insert_no_replace(&mut self, value: T) -> (T, usize) {
    // TODO: Same questions as in `contains`.
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    let key = fast_hasher.finish();
    let mut entry = self.inner.entry(key);

    match entry {
      Entry::Vacant(vacant_entry) => {
        let index = self.hashes.len();
        self.hashes.push(key);

        vacant_entry.insert((value.clone(), index));
        (value, index)
      }

      Entry::Occupied(mut occupied_entry) => {
        let (found, idx): &(T, usize) = occupied_entry.get();
        (found.clone(), *idx)
      }
    } // end match
  }

  /// Fetches the value from the set, returning `None` if it is not present.
  #[inline(always)]
  pub fn find_for_hash(&self, hash: u64) -> Option<(T, usize)> {
    self.inner.get(&hash).cloned()
  }

  /// Fetches the value from the set, returning `None` if it is not present.
  #[inline(always)]
  pub fn find_for_index(&self, index: usize) -> Option<(T, usize)> {
    if let Some(key) = self.hashes.get(index) {
      self.inner.get(key).cloned()
    } else {
      None
    }
  }

  #[inline(always)]
  fn compute_hash(&self, value: &T) -> u64 {
    // TODO: What is the best way to do this? Use `self.inner.hasher()`? Call `value.hash(..)` or
    //       `value.compute_hash()`? Also, should `compute_hash()` return a `u32` or `u64`?
    let mut fast_hasher = self.inner.hasher().build_hasher();
    value.hash(&mut fast_hasher);

    fast_hasher.finish()
  }
}

// In this impl, `T = Rc<U>`
impl<U: Hash> IndexedHashSet<Rc<U>> {
  pub fn contains(&self, value: Rc<U>) -> bool
  // where
  //   U: Borrow<Q> + Hash,
    // Q: Hash + Eq + ?Sized,
  {
    let key = self.compute_hash(&value);
    self.inner.contains_key(&key)
  }

  /// Finds the provided (borrowed) term, if it is in the set.
  pub fn find(&self, value: Rc<U>) -> Option<(Rc<U>, usize)>
  // where
  //   U: Borrow<Q>,
  //   Q: Hash + Eq + ?Sized,
  {
    let key = self.compute_hash(&value);
    self.find_for_hash(key)
  }
}
