/*!

Types/type aliases that abstract over the implementing backing type.

A motivating example is the `RcCell` type, a reference-counting smart pointer that provides run-time checked mutable
access to its contents and supports weak references. A number of external crates could provide this functionality. This
module redirects to whatever chosen implementation we want.

*/
mod graph;
mod hash;
mod hash_set;
mod indexed_hash_set;
mod nat_set;
mod rccell;

use std::{collections::HashSet as StdHashSet, iter::once};

pub use graph::Graph;
// Fast and simple hash functions
pub use hash::{hash2, hash3, FastHasher, FastHasherBuilder};
// A hash set of terms for structural sharing
pub use hash_set::{
  // DagNodeHashSet,
  HashSet,
  HashValueType,
  TermHashSet
};
// Similar to a HashSet, except an index equal to original insertion order is assigned to each element
pub use indexed_hash_set::IndexedHashSet;
// A set of natural numbers
pub use nat_set::NatSet;
// Reference counted pointers with mutable stable, and complementary weak pointers.
pub use rccell::{rc_cell, RcCell, WeakCell};
// Interned string.
pub use string_cache::DefaultAtom as IString;
pub use tiny_logger::{log, set_verbosity, Channel};
pub use yansi::{Color, Paint, Style};

/// Arbitrary precision integers
pub type BigInteger = isize; // ToDo: An `isize` is not "arbitrary precision."

/// A `ThingSet` is a hash set of `*const dyn Things`. They are useful if you need to test membership but never need
/// to access the original `Thing`.
pub type Set<T> = StdHashSet<*const T>; // This replaces Maude's `PointerSet` in most situations.

/**

Join an iterator of strings, which doesn't exist in the stdlib. (C.f. `Vec::join(…)`)

From: https://stackoverflow.com/a/66951473

Usage:

    let iter = [1, 3, 5, 7, 9].iter().cloned();
    println!("{:?}", join_iter(iter, |v| v - 1).collect::<Vec<_>>());
    // [1, 2, 3, 4, 5, 6, 7, 8, 9]

    let iter = ["Hello", "World"].iter().cloned();
    let sep = ", ";
    println!("{:?}", join_iter(iter, |_| sep).collect::<String>());
    // "Hello, World"
*/
pub fn join_iter<T>(mut iter: impl Iterator<Item = T>, sep: impl Fn(&T) -> T) -> impl Iterator<Item = T> {
  iter
    .next()
    .into_iter()
    .chain(iter.flat_map(move |s| once(sep(&s)).chain(once(s))))
}
