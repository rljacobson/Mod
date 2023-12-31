/*

This module houses things related to global state, the execution environment, formatting output, and tracing execution.

*/

mod attributes;
mod print_flags;
pub mod module;
pub mod rewrite_context;
mod interpreter_state;
pub mod format;
mod tui;
// pub mod memo_table;

pub use attributes::{InterpreterAttribute, InterpreterAttributes};
pub use print_flags::{PrintFlag, PrintFlags};
pub use interpreter_state::{Interpreter, SearchKind, WeakInterpreter};


pub type ContinueFuncPtr = fn(&mut Interpreter, limit: usize, debug: bool);
pub type SourceSet = Vec<i32>;

