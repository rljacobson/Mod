use std::{
  collections::HashSet,
  rc::{Rc, Weak}
};

use crate::{
  abstractions::IString,
  core::{
    CacheableState,
    SyntacticPreModule,
    SyntacticView,
    Token,
    VisibleModule
  }
};

use super::{
  ContinueFuncPtr,
  InterpreterAttribute,
  InterpreterAttributes,
  PrintFlags
};


pub type RcInterpreter = Rc<Interpreter>;
pub type WeakInterpreter = Weak<Interpreter>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum SearchKind {
  #[default]
  Search,
  Narrow,
  XGNarrow,
  SMTSearch,
  VUNarrow,
  FVUNarrow,
}

#[derive()]
pub struct Interpreter {

  // ToDo: We won't be implementing MaudeML, but what kind of logging do we want?
  // xml_log   : Option<File>, // Unused?
  // xml_buffer: Option<MaudemlBuffer>,

  attributes    : InterpreterAttributes,
  print_flags   : PrintFlags,
  current_module: Option<SyntacticPreModule>,
  current_view  : Option<SyntacticView>,

  // Continuation information
  saved_state         : Option<CacheableState>,
  saved_solution_count: u64,                     // ToDo: As far as I know, this is nonnegative, so changed i64->u64.
  saved_module        : Option<VisibleModule>,   // ToDo: Why is this a different type from `current_module`?
  continue_func       : Option<ContinueFuncPtr>,
  saved_loop_subject  : Vec<Token>,              // ToDo: Why is the loop subject a syntactic structure?

  // ToDo: These objects are all referenced by _name_. Should they instead be referenced by index or something else?
  selected         : HashSet<IString>, // Temporary for building set of identifiers
  trace_names      : HashSet<IString>, // Names of symbols/labels selected for tracing
  pub(crate) break_names: HashSet<IString>, // Names of symbols/labels selected as break points
  excluded_modules : HashSet<IString>, // Names of modules to be excluded from tracing
  concealed_symbols: HashSet<IString>, // Names of symbols to have their arguments concealed during printing
}

impl Interpreter{
  pub fn attribute(&self, attribute: InterpreterAttribute) -> bool {
    self.attributes.has_attribute(attribute)
  }

  pub fn trace_name(&self, name: &IString) -> bool {
    self.trace_names.contains(name)
  }

  pub fn excluded_module(&self, name: &IString) -> bool {
    self.excluded_modules.contains(name)
  }
}
