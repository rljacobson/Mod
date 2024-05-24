/*!

A `Module` serves as a kind of symbol table and holds information local to a module. It's the equivalent of a
translation unit in C++ or a module in Python or Rust.

`ModuleItem`s are objects that are numbered within a module. This provides us with:
  (1) a way of getting back to the module containing an object; and
  (2) a number that is useful for indexing.

*/

pub(crate) mod item;
// mod memo_map;
pub(crate) mod module;
mod profile;

pub use module::Module;
pub use profile::{FragmentProfile, StatementProfile, SymbolProfile};

use crate::abstractions::WeakCell;

pub type WeakModule = WeakCell<Module>;
