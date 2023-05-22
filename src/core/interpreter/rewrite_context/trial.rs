use std::rc::Rc;

use crate::{
  abstractions::NatSet,
  core::{
    Equation,
    format::{FormatStyle, Formattable},
    interpreter::{
      Interpreter,
      interpreter_state::RcInterpreter,
      InterpreterAttribute,
    },
    module::{Module, ModuleItem, StatementProfile},
    pre_equation::PreEquation,
    rewrite_context::{
      context_attributes::{ContextAttribute, ContextAttributes}, ,
      HEADER,
      RewritingContext,
    },
    rule::Rule,
    sort::SortConstraint,
    StrategyDefinition,
    substitution::print_substitution
  },
  theory::{DagNode, RcDagNode}
};

use super::debugger;


impl RewritingContext {
  /*fn trace_begin_eq_trial(&mut self, subject: RcDagNode, equation: &Equation) -> Option<i32> {
    // assert!(equation != 0, "null equation in trial");

    let interpreter: RcInterpreter = self.interpreter.upgrade().unwrap();

    if interpreter.attribute(InterpreterAttribute::Profile) {
      let module: &mut Module = &mut *self.root
          .borrow()
          .symbol()
          .get_module()
          .upgrade()
          .unwrap()
          .borrow_mut();
          // .profile_eq_condition_start(equation);
      module.profile_condition_start(equation, &mut module.eq_info);
    }
    if self.handle_debug(subject.clone(), equation) {
      return None;
    }
    if !self.attribute(ContextAttribute::LocalTrace)
        || !interpreter.attribute(InterpreterAttribute::TraceEq)
        || self.do_not_trace(subject.clone(), equation)
    {
      return None;
    }
    println!("{}trial #{}\n{}", HEADER, self.trial_count + 1, equation.repr(FormatStyle::Default));
    if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
      print_substitution(&self.substitution, equation.variable_info(), &NatSet::new());
    }
    Some((self.trial_count + 1) as i32)
  }*/

  pub(crate) fn trace_begin_trial(
    &mut self,
    subject: RcDagNode,
    pre_equation: &dyn PreEquation,           // &Equation
    trace_attribute: InterpreterAttribute,    // InterpreterAttribute::TraceEq
  ) -> Option<i32>
  {
    // assert!(equation != 0, "null equation in trial");

    let interpreter: RcInterpreter = self.interpreter.upgrade().unwrap();

    if interpreter.attribute(InterpreterAttribute::Profile) {
      let module: &mut Module = &mut *self.root
          .borrow()
          .symbol()
          .get_module()
          .upgrade()
          .unwrap()
          .borrow_mut();
      module.profile_condition_start(pre_equation, trace_attribute);

    }

    if self.handle_debug(subject.clone(), pre_equation) {
      return None;
    }

    if !self.attribute(ContextAttribute::LocalTrace)
        || !interpreter.attribute(trace_attribute)
        || self.do_not_trace(subject.clone(), pre_equation)
    {
      return None;
    }

    println!("{}trial #{}\n{}", HEADER, self.trial_count + 1, pre_equation.repr(FormatStyle::Default));
    if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
      print_substitution(&self.substitution, pre_equation.variable_info(), &NatSet::new());
    }

    Some((self.trial_count + 1) as i32)
  }

  /*
  pub(crate) fn trace_begin_rule_trial(&mut self, subject: RcDagNode, rule: &Rule) -> Option<i32> {
    // assert!(rule != 0, "null rule in trial");

    if self.interpreter.attribute(InterpreterAttribute::Profile) {
      self.root
          .borrow()
          .symbol()
          .get_module()
          .upgrade()
          .unwrap()
          .borrow_mut()
          .profile_rl_condition_start(rule);
    }

    if self.handle_debug(subject, rule) {
      return None;
    }

    if !self.attribute(ContextAttribute::LocalTrace)
        || !interpreter.attribute(InterpreterAttribute::TraceRl)
        || self.do_not_trace(subject.clone(), rule)
    {
      return None;
    }

    println!("{}trial #{}\n{}", self.header, self.trial_count + 1, rule.repr(FormatStyle::Default));
    if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
      self.print_substitution(&self.substitution, rule);
    }

    Some(self.trial_count as i32 + 1)
  }

  fn trace_begin_sc_trial(&mut self, subject: RcDagNode, sort_constraint: &SortConstraint) -> Option<i32> {
    assert!(sort_constraint != 0, "null membership axiom in trial");
    if interpreter.attribute(InterpreterAttribute::Profile) {
      self.root
          .borrow()
          .symbol()
          .get_module()
          .unwrap()
          .profile_mb_condition_start(sort_constraint);
    }
    if self.handle_debug(subject.clone(), sort_constraint) {
      return None;
    }
    if !self.attribute(ContextAttribute::LocalTrace)
        || !interpreter.attribute(InterpreterAttribute::TraceMb)
        || self.do_not_trace(subject.clone(), sort_constraint)
    {
      return None;
    }
    println!("{}trial #{}\n{}", self.header, self.trial_count + 1, sort_constraint);
    if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
      self.print_substitution(&self.substitution, sort_constraint);
    }
    Some((self.trial_count + 1) as i32)
  }

  fn trace_begin_sd_trial(&mut self, subject: RcDagNode, strategy_definition: &StrategyDefinition) -> Option<i32> {
    assert!(strategy_definition != 0, "null strategy definition in trial");
    if interpreter.attribute(InterpreterAttribute::Profile) {
      self.root
          .borrow()
          .symbol()
          .get_module()
          .upgrade()
          .unwrap()
          .profile_sd_condition_start(strategy_definition);
    }
    if self.handle_debug(subject.clone(), strategy_definition) {
      return None;
    }
    if !self.attribute(ContextAttribute::LocalTrace)
        || !interpreter.attribute(InterpreterAttribute::TraceSd)
        || self.do_not_trace(subject.clone(), strategy_definition)
    {
      return None;
    }
    println!("{}trial #{}\n{}", self.header, self.trial_count + 1, strategy_definition);
    if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
      self.print_substitution(&self.substitution, strategy_definition);
    }

    Some((self.trial_count + 1) as i32)
  }
*/

  /// Debugger.
  // ToDo: This should live in the interpreter, shouldn't it?
  fn handle_debug(&mut self, subject: RcDagNode, pe: &dyn PreEquation) -> bool {
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
      // TODO: Unimplemented
      // self.print_status_report(subject, pe);
      // TODO: Why are we setting this flag here when we just checked that it was set?
      self.attributes.set(ContextAttribute::Info);

      // If we are only slow routed by an INFO signal we want
      // to make sure we take the fast route now that we've made
      // our report.
      self.attributes.set_trace_status(interpreter.attribute(InterpreterAttribute::ExceptionFlags));
    }

    let mut broken = false;
    let mut broken_symbol = None;
    if interpreter.attribute(InterpreterAttribute::Break) {
      let symbol = subject.borrow().symbol();
      if interpreter.break_names.contains(&symbol.name()) {
        broken = true;
        broken_symbol = Some(symbol);
      }
      else //if let Some(pe) = pe
        if interpreter.break_names.contains(&pe.name().unwrap()) {
          broken = true;
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

    }
    else if let Some(broken_symbol) = broken_symbol {

      println!("break on symbol: {}", broken_symbol.repr(FormatStyle::Default));

    }
    // else if let Some(pe) = pe
    else if let Some(mb) = pe.as_any().downcast_ref::<SortConstraint>() {
      println!("break on labeled membership axiom:\n{}", mb);
    } else if let Some(eq) = pe.as_any().downcast_ref::<Equation>() {
      println!("break on labeled equation:\n{}", eq);
    } else if let Some(rl) = pe.as_any().downcast_ref::<Rule>() {
      println!("break on labeled rule:\n{}", rl);
    } else if let Some(sdef) = pe.as_any().downcast_ref::<StrategyDefinition>() {
      println!("break on labeled strategy definition:\n{}", sdef);
    } else {
      CantHappen("unidentified statement");
    }

    self.step_flag = false;
    self.set_trace_status(self.interpreter.attribute(InterpreterAttribute::ExceptionFlags));
    loop {
      match self.command_loop() {
        Command::RESUME => {
          self.debug_level -= 1;
          self.change_prompt();
          return !interpreter.attribute(Interpreter::TRACE);
        }
        Command::ABORT => {
          self.debug_level -= 1;
          self.change_prompt();
          self.attribute(ContextAttributes::Abort) = true;
          self.set_trace_status(true);
          return true;
        }
        Command::STEP => {
          self.debug_level -= 1;
          self.change_prompt();
          self.step_flag = true;
          self.set_trace_status(true);
          return false;
        }
        Command::WHERE => {
          self.where_(std::io::stdout().lock());
        }
      }
    }
    true // never executed
  }



}
