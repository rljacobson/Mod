/*

This module houses things related to global state, the execution environment, formatting output, and tracing execution.

*/

mod attributes;
pub mod format;
mod interpreter_state;
pub mod module;
mod print_flags;
pub mod rewrite_context;
mod tui;
// pub mod memo_table;

pub use attributes::{InterpreterAttribute, InterpreterAttributes};
pub use interpreter_state::{Interpreter, SearchKind, WeakInterpreter};
pub use print_flags::{PrintFlag, PrintFlags};


pub type ContinueFuncPtr = fn(&mut Interpreter, limit: usize, debug: bool);
pub type SourceSet = Vec<i32>;
