/*!

Debugger related code. This is a lot more of Maude's infrastructure than really should be in this library, but I will
need to understand the boundary between algorithms and supporting code better before I refactor.

*/

use crate::{
  core::{
    format::{FormatStyle, Formattable},
    interpreter::{interpreter_state::RcInterpreter, tui::DEFAULT_PROMPT, InterpreterAttribute},
    pre_equation::PreEquation,
    rewrite_context::{context_attributes::ContextAttribute, RewritingContext},
  },
  theory::RcDagNode,
};

/// Result of parsing the debugger command line.
pub enum ParseResult {
  Normal,
  Quit,
  Resume,
  Abort,
  Step,
  Where,
}


impl RewritingContext {
  pub fn change_prompt(&mut self) {
    if self.debug_level == 0 {
      self.tui.set_prompt(DEFAULT_PROMPT.to_string());
      self.attributes.reset(ContextAttribute::DebugMode);
    } else {
      let prompt = format!("Debug({})> ", self.debug_level);
      self.tui.set_prompt(prompt);
      self.attributes.set(ContextAttribute::DebugMode);
    }
  }

  /// Debugger.
  // ToDo: This should live in the interpreter, shouldn't it?
  pub(crate) fn handle_debug(&mut self, subject: RcDagNode, pre_equation: Option<&PreEquation>) -> bool {
    // Handle unusual situations that are common to all rewrite types:
    // (a) Abort
    // (b) Info interrupt
    // (c) Breakpoints
    // (d) ^C interrupt
    // (e) Single stepping
    // In the latter 3 cases, we drop into the debugger.
    if self.attribute(ContextAttribute::Abort) {
      return true;
    }

    let interpreter: RcInterpreter = self.interpreter.upgrade().unwrap();

    if self.attribute(ContextAttribute::Info) {
      // self.print_status_report(subject, pe); // TODO: Unimplemented
      // TODO: Why are we setting this flag here when we just checked that it was set?
      self.attributes.set(ContextAttribute::Info);

      // If we are only slow routed by an INFO signal we want to make sure we take the fast route
      // now that we've made our report.
      self
        .attributes
        .set_trace_status(interpreter.attribute(InterpreterAttribute::ExceptionFlags));
    }

    let mut broken = false;
    let mut broken_symbol = None;
    if interpreter.attribute(InterpreterAttribute::Break) {
      let symbol = subject.borrow().symbol();
      if interpreter.break_names.contains(&symbol.name()) {
        broken = true;
        broken_symbol = Some(symbol);
      } else if let Some(pe) = pre_equation {
        if interpreter.break_names.contains(&pe.name.unwrap()) {
          broken = true;
        }
      }
    }

    if !(self.attribute(ContextAttribute::CtrlC) || self.attribute(ContextAttribute::Step) || broken) {
      return !interpreter.attribute(InterpreterAttribute::Trace); // normal tracing
    }

    self.debug_level += 1;
    self.change_prompt();

    if self.attribute(ContextAttribute::CtrlC) {
      if !self.attribute(ContextAttribute::Interactive) {
        println!();
        // Close all files & modules.
        // TODO: Unimplemented
        // self.clean_up_lexer();
      }
      self.attributes.reset(ContextAttribute::CtrlC);
    } else if let Some(broken_symbol) = broken_symbol {
      println!("break on symbol: {}", broken_symbol.repr(FormatStyle::Default));
    } else {
      if let Some(pre_equation) = pre_equation {
        println!(
          "break on labeled {}:\n{}",
          pre_equation.kind.noun(),
          pre_equation.repr(FormatStyle::Simple)
        );
      } else {
        println!("break on unknown statement");
      }
    }

    self.attributes.reset(ContextAttribute::Step);
    self
      .attributes
      .set_trace_status(interpreter.attribute(InterpreterAttribute::ExceptionFlags));
    loop {
      match self.tui.command_loop() {
        ParseResult::Resume => {
          self.debug_level -= 1;
          self.change_prompt();
          return !interpreter.attribute(InterpreterAttribute::Trace);
        }
        ParseResult::Abort => {
          self.debug_level -= 1;
          self.change_prompt();
          self.attributes.set(ContextAttribute::Abort);
          self.attributes.set_trace_status(true);
          return true;
        }
        ParseResult::Step => {
          self.debug_level -= 1;
          self.change_prompt();
          self.attributes.set(ContextAttribute::Step);
          self.attributes.set_trace_status(true);
          return false;
        }
        ParseResult::Where => {
          // self.where_(std::io::stdout().lock());
          // ToDo: What is the equivalent?
          return false;
        }
        _ => {
          unreachable!()
        }
      }
    }
    // unreachable!() // never executed
  }
}
