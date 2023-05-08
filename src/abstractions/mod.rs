/*!

Types/type aliases that abstract over the implementing backing type.

A motivating example is the `RcCell` type, a reference-counting smart pointer that provides run-time checked mutable
access to its contents and supports weak references. A number of external crates could provide this functionality. This
module redirects to whatever chosen implementation we want.

*/

mod rc_cell;
mod hash;

use std::collections::HashSet;


// Interned string.
pub use string_cache::DefaultAtom as IString;
// Reference counted pointers with mutable stable, and complementary weak pointers.
pub use rc_cell::{RcCell, WeakCell};
pub use hash::{hash2, hash3, FastHasher, FastHasherBuilder};


/// Arbitrary precision integers
pub type BigInteger = isize;
/// A `ThingSet` is a hash set of `*const dyn Things`. They are useful if you need to test membership but never need
/// to access the original `Thing`.
pub type Set<T> = HashSet<*const T>; // This replaces Maude's `PointerSet` in most situations.
