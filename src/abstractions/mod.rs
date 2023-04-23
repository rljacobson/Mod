/*!

Types/type aliases that abstract over the implementing backing type.

A motivating example is the `RcCell` type, a reference-counting smart pointer that provides run-time checked mutable
access to its contents and supports weak references. A number of external crates could provide this functionality. This
module redirects to whatever chosen implementation we want.

*/

mod rc_cell;

/// Interned string.
pub use string_cache::DefaultAtom as IString;
/// Reference counted pointers with mutable stable, and complementary weak pointers.
pub use rc_cell::{RcCell, WeakCell};
/// Arbitrary precision integers
pub type BigInteger = isize;
