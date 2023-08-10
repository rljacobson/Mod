/*!

A thin wrapper around BitSet (the bit-set crate). We could just use a type alias if we didn't also need a `min` method.

*/

use bit_set::BitSet;
pub use bit_set::{Iter as BitSetIterator};

#[derive(Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NatSet(BitSet<u32>);

pub type NatSetIterator<'a> = BitSetIterator<'a, u32>;

impl NatSet {

  /*
    capacity
      clear
      contains
      difference
      difference_with
    from_bit_vec
    from_bytes
    get_ref
      insert
      intersect_with
      intersection
    into_bit_vec
      is_disjoint
      is_empty
      is_subset
      is_superset
      iter
      len
      new
      remove
      reserve_len
      reserve_len_exact
      shrink_to_fit
      symmetric_difference
      symmetric_difference_with
      union
      union_with
      with_capacity
  */

  #[inline(always)]
  pub fn clear(&mut self) {
    self.0.clear()
  }

  #[inline(always)]
  pub fn min_value(&self) -> Option<usize> {
    self.0.iter().next()
  }

  #[inline(always)]
  pub fn contains(&self, value: usize) -> bool {
    self.0.contains(value)
  }

  /// Returns the difference with the other specified bit vector.
  #[inline(always)]
  pub fn difference(&self, other: &NatSet) -> NatSet {
    let mut new_set = self.clone();
    new_set.0.difference_with(&other.0);
    new_set
  }

  /// Makes this bit vector the difference with the specified other bit vector in-place.
  #[inline(always)]
  pub fn difference_in_place(&mut self, other: &NatSet) {
    self.0.difference_with(&other.0);
  }

  /// Adds a value to the set. Returns true if the value was not already present in the set.
  #[inline(always)]
  pub fn insert(&mut self, value: usize) -> bool {
    self.0.insert(value)
  }

  /// Returns the intersection with the other specified bit vector.
  #[inline(always)]
  pub fn intersection(&self, other: &NatSet) -> NatSet {
    let mut new_set = self.clone();
    new_set.0.intersect_with(&other.0);
    new_set
  }

  #[inline(always)]
  pub fn is_disjoint(&self, other: &NatSet) -> bool {
    self.0.is_disjoint(&other.0)
  }

  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  #[inline(always)]
  pub fn is_subset(&self, other: &NatSet) -> bool {
    self.0.is_subset(&other.0)
  }

  #[inline(always)]
  pub fn is_superset(&self, other: &NatSet) -> bool {
    self.0.is_superset(&other.0)
  }

  #[inline(always)]
  pub fn iter(&self) -> NatSetIterator {
    self.0.iter()
  }

  #[inline(always)]
  pub fn len(&self) -> usize {
    self.0.len()
  }

  #[inline(always)]
  pub fn new() -> Self {
    Self::default()
  }

  /// Removes a value from the set. Returns true if the value was present in the set.
  #[inline(always)]
  pub fn remove(&mut self, value: usize) -> bool {
    self.0.remove(value)
  }

  #[inline(always)]
  pub fn reserve_len(&mut self, len: usize) {
    self.0.reserve_len(len)
  }

  #[inline(always)]
  pub fn reserve_len_exact(&mut self, len: usize) {
    self.0.reserve_len_exact(len)
  }

  #[inline(always)]
  pub fn shrink_to_fit(&mut self) {
    self.0.shrink_to_fit()
  }

  /// Returns the symmetric difference with the other specified bit vector.
  #[inline(always)]
  pub fn symmetric_difference(&self, other: &NatSet) -> NatSet {
    let mut new_set = self.clone();
    new_set.0.symmetric_difference(&other.0);
    new_set
  }

  /// Returns the symmetric difference with the other specified bit vector.
  #[inline(always)]
  pub fn symmetric_difference_in_place(&mut self, other: &NatSet) {
    self.0.symmetric_difference(&other.0);
  }

  /// Returns the union with the other specified bit vector.
  #[inline(always)]
  pub fn union(&self, other: &NatSet) -> NatSet {
    let mut new_set = self.clone();
    new_set.0.union_with(&other.0);
    new_set
  }

  /// Makes this bit vector the union with the specified other bit vector in-place.
  #[inline(always)]
  pub fn union_in_place(&mut self, other: &NatSet) {
    self.0.union_with(&other.0);
  }

  #[inline(always)]
  pub fn with_capacity(nbits: usize) -> NatSet {
    NatSet(BitSet::with_capacity(nbits))
  }

}
