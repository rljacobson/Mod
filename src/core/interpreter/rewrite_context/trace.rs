use std::sync::atomic::{AtomicBool, Ordering};
use yansi::Paint;

use crate::{
  abstractions::{
    Channel,
    join_iter,
    log,
    NatSet,
  },
  core::{
    Equation,
    format::{FormatStyle, Formattable},
    interpreter::InterpreterAttribute,
    module::ModuleItem,
    NarrowingVariableInfo,
    pre_equation::PreEquation,
    rule::Rule,
    sort::SortConstraint,
    StrategyDefinition,
    substitution::{
      MaybeDagNode,
      print_substitution,
      print_substitution_narrowing,
      Substitution
    },
    Token,
  },
  NONE,
  theory::{DagNode, RcDagNode},
};

use super::{
  RewritingContext,
  HEADER,
  RewriteType
};


/// Tracing status is global for all `RewritingContext`s.
static TRACE_STATUS: AtomicBool = AtomicBool::new(false);

// ToDo: Make trace_status local to context or interpreter
/// Tracing status is global for all `RewritingContext`s.
pub fn trace_status() -> bool {
  TRACE_STATUS.load(Ordering::Relaxed)
}
/// Tracing status is global for all `RewritingContext`s.
pub fn set_trace_status(status: bool) {
  TRACE_STATUS.store(status, Ordering::Relaxed);
}


impl RewritingContext {

  pub fn do_not_trace(&self, redex: RcDagNode, pe: &dyn PreEquation) -> bool {
    let symbol = redex.borrow().symbol();
    let interpreter = self.interpreter.upgrade().unwrap();
    (interpreter.attribute(InterpreterAttribute::TraceSelect)
        && !(interpreter.trace_name(&symbol.name())
        || (pe.name().is_some() && interpreter.trace_name(&pe.name().unwrap()))))
        || interpreter.excluded_module(&symbol.get_module().upgrade().unwrap().borrow().name)
    // ToDo: Determine where the following is used.
        // || (pe.is_none() && !interpreter.attribute(InterpreterAttribute::TraceBuiltin))
  }

  /* Print attributes are unimplemented.
  pub fn check_for_print_attribute(
    &self,
    item_type: ItemType,
    item: Option<&dyn PreEquation>,
  ) {
    if let Some(item) = item {
      let module = item.get_module();
      if let Some(pa) = module.print_attribute(item_type, item) {
        pa.print(io::stdout()).unwrap();
        if Interpreter::attribute(InterpreterAttribute::PrintAttribute_NEWLINE) {
          println!();
        }
      }
    }
  }
  */

  pub fn trace_pre_eq_rewrite(&mut self, redex: RcDagNode, equation: Option<&Equation>, eq_type: RewriteType) {
    // All unusual situations during an equational rewrite are funneled
    // through this function, by setting the traceFlag in class Module.
    // This is so that rewriting only has to test a single flag
    // to get off the fast case, and into the (slow) exception case.
    //
    // Possible unusual situations:
    // (1) Profiling is enabled
    // (2) Statement print attributes are enabled
    // (3) Aborting the computation
    // (4) Single stepping in debugger
    // (5) Breakpoints are enabled
    // (6) ^C interrupt
    // (7) Info interrupt
    // (8) Tracing is enabled

    let redex_ref: &dyn DagNode = redex.borrow().into();
    let interpreter = self.interpreter.upgrade().unwrap();

    if interpreter.attribute(InterpreterAttribute::Profile) {
      let profile_module = self.root().symbol().get_module();
      profile_module.profile_eq_rewrite(redex.clone(), equation, eq_type);
    }
    // if interpreter.attribute(InterpreterAttribute::PrintAttribute) {
    //   self.check_for_print_attribute(MetadataStore::EQUATION, equation);
    // }

    // handeDebug() takes care of abort, single stepping, breakpoints,
    // ^C interrupts and info interrupts. These are common to
    // all rewrite types.
    if self.handle_debug(redex.clone(), equation)
        || !self.local_trace_flag
        || !interpreter.attribute(InterpreterAttribute::TraceEq)
        || self.do_not_trace(redex, equation)
    {
      self.trace_post_flag = false;
      return;
    }
    self.trace_post_flag = true;

    if interpreter.attribute(InterpreterAttribute::TraceBody) {
      println!("{}", HEADER);
      println!("equation");
    }

    if let Some(equation) = equation {
      if interpreter.attribute(InterpreterAttribute::TraceBody) {
        println!("{}", equation.repr());

        if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
          print_substitution(&self.substitution, equation.variable_info(), &NatSet::default());
        }
      } else {
        if let Some(label) = equation.name(){
          println!("{}", label);
        } else {
          println!("(unlabeled equation)");
        }
      }

    } else {

      if eq_type == RewriteType::Builtin {
        println!("(built-in equation for symbol {})", redex_ref.symbol().repr(FormatStyle::Simple));
      }
      else if eq_type == RewriteType::Memoized {
        println!("(memo table lookup for symbol {})", redex_ref.symbol().repr(FormatStyle::Simple));
      }
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      println!("Old: {}", self.root());
    }

    if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
      println!("{} \n--->", redex_ref);
    }

    log(Channel::Debug, 1, redex_ref);
  }

  pub fn trace_post_eq_rewrite(&self, replacement: RcDagNode) {
    if self.trace_post_flag {
      assert!(!self.abort_flag, "abort flag set");
      let interpreter = self.interpreter.upgrade().unwrap();

      if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
        println!("{}", replacement.borrow().to_string());
      }

      log(Channel::Debug, 1, replacement.borrow().to_string().as_str());

      if interpreter.attribute(InterpreterAttribute::TraceWhole) {
        println!("New: {}", self.root());
      }
    }
  }


  pub fn trace_pre_rule_rewrite(&mut self, redex: MaybeDagNode, rule: Option<&Rule>) {
    if redex.is_none() {
      // Dummy rewrite; need to ignore the following trace_post_rule_rewrite() call.
      self.trace_post_flag = false;
      return;
    }

    let interpreter = self.interpreter.upgrade().unwrap();
    let redex = redex.unwrap();

    if interpreter.attribute(InterpreterAttribute::Profile) {
      let mut profile_module = self.root.borrow().symbol().get_module().upgrade().unwrap().borrow();
      profile_module.profile_rl_rewrite(redex.clone(), rule);
    }

    // if interpreter.attribute(InterpreterAttribute::PrintAttribute) {
    //   self.check_for_print_attribute(MetadataStore::RULE, rule);
    // }

    if self.handle_debug(redex.clone(), rule)
        || !self.local_trace_flag
        || !interpreter.attribute(InterpreterAttribute::TraceRl)
        || self.do_not_trace(redex.clone(), rule)
    {
      self.trace_post_flag = false;
      return;
    }
    self.trace_post_flag = true;

    if interpreter.attribute(InterpreterAttribute::TraceBody) {
      println!("{} rule", HEADER);
    }

    if let Some(rule) = rule {
      if interpreter.attribute(InterpreterAttribute::TraceBody) {
        println!("{}", rule.repr(FormatStyle::Default));
        if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
          print_substitution_narrowing(&self.substitution, rule.variable_info);
        }
      } else {
        if let Some(label) = rule.name(){
          println!("{}", label);
        } else {
          println!("(unlabeled rule)");
        }
      }
    } else {
      println!("(built-in rule for symbol {})", redex.symbol());
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      println!("Old: {}", self.root());
    }

    if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
      println!("{} \n--->", redex.borrow());
    }
  }

  pub fn trace_post_rule_rewrite(&self, replacement: RcDagNode) {
    if self.trace_post_flag {
      let interpreter = self.interpreter.upgrade().unwrap();

      if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
        println!("{}", replacement.borrow());
      }

      if interpreter.attribute(InterpreterAttribute::TraceWhole) {
        println!("New: {}", self.root());
      }
    }
  }


  pub fn trace_narrowing_step(
    &self,
    rule: &Rule,
    redex: RcDagNode,
    replacement: RcDagNode,
    variable_info: &NarrowingVariableInfo,
    substitution: &Substitution,
    new_state: RcDagNode,
  ) {
    let interpreter = self.interpreter.upgrade().unwrap();
    if self.handle_debug(redex.clone(), rule)
        || !self.local_trace_flag
        || !interpreter.attribute(InterpreterAttribute::TraceRl)
        || self.do_not_trace(redex.clone(), Some(rule))
    {
      return;
    }

    if interpreter.attribute(InterpreterAttribute::TraceBody) {
      println!("{}", Paint::magenta("narrowing step"));
      println!("{}", rule.repr(FormatStyle::Simple));

      if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
        println!("Rule variable bindings:");
        print_substitution(substitution, &rule.variable_info, &NatSet::default());
        println!("Subject variable bindings:");

        let subject_variable_count = variable_info.variable_count();
        if subject_variable_count == 0 {
          println!("empty substitution");
        } else {
          let variable_base = rule.get_module().get_minimum_substitution_size();
          for i in 0..subject_variable_count {
            let v = variable_info.index2variable(i);
            let d = substitution.value(variable_base + i);

            assert!(v != 0, "null variable");
            print!("{} --> ", v);

            if let Some(d) = d {
              println!("{}", d.borrow());
            } else {
              println!("(unbound)");
            }
          }
        }
      }
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      println!("Old: {}", self.root());
    }

    if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
      println!("{} \n--->\n{}", redex.borrow(), replacement.borrow());
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      println!("New: {}", new_state.borrow());
    }
  }

  pub fn trace_variant_narrowing_step(
    &self,
    equation: &Equation,
    old_variant_substitution: &[RcDagNode],
    redex: RcDagNode,
    replacement: RcDagNode,
    variable_info: &NarrowingVariableInfo,
    substitution: &Substitution,
    new_state: RcDagNode,
    new_variant_substitution: &[RcDagNode],
    original_variables: &NarrowingVariableInfo,
  ) {
    let interpreter = self.interpreter.upgrade().unwrap();
    if self.handle_debug(redex.clone(), equation)
        || !self.local_trace_flag
        || !interpreter.attribute(InterpreterAttribute::TraceEq)
        || self.do_not_trace(redex.clone(), Some(equation))
    {
      return;
    }

    if interpreter.attribute(InterpreterAttribute::TraceBody) {
      println!(" {}", Paint::cyan("variant narrowing step"));
      println!("{}", equation.repr(FormatStyle::Simple));

      if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
        println!("Equation variable bindings:");
        print_substitution(substitution, equation.variable_info(), &NatSet::default());
        println!("Old variant variable bindings:");

        let subject_variable_count = variable_info.variable_count();
        if subject_variable_count == 0 {
          println!("empty substitution");
        } else {
          let variable_base = equation.get_module().get_minimum_substitution_size();
          for i in 0..subject_variable_count {
            let v = variable_info.index2variable(i);
            let d = substitution.value(variable_base + i);

            assert!(v != 0, "null variable");
            print!("{} --> ", v);

            if let Some(d) = d {
              println!("{}", d.borrow());
            } else {
              println!("(unbound)");
            }
          }
        }
      }
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      println!("\nOld variant: {}", self.root());
      print_substitution_narrowing(old_variant_substitution, original_variables);
      println!();
    }

    if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
      println!("{} \n--->\n{}", redex.borrow(), replacement.borrow());
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      println!("\nNew variant: {}", new_state.borrow());
      print_substitution_narrowing(new_variant_substitution, original_variables);
      println!();
    }
  }


  pub fn trace_strategy_call(
    &self,
    sdef: &StrategyDefinition,
    call_dag: RcDagNode,
    subject: RcDagNode,
    substitution: &Substitution,
  ) {
    let interpreter = self.interpreter.upgrade().unwrap();
    if interpreter.attribute(InterpreterAttribute::Profile) {
      if let Some(profile_module)
          = self.root
          .borrow()
          .symbol()
          .get_module()
          .upgrade()
          .unwrap()
          .borrow() {
        profile_module.profile_sd_rewrite(subject, sdef);
      }
    }
    // if interpreter.attribute(InterpreterAttribute::PrintAttribute) {
    //   check_for_print_attribute(MetadataStore::STRAT_DEF, sdef);
    // }

    if self.handle_debug(call_dag.clone(), sdef)
        || !self.local_trace_flag
        || !interpreter.attribute(InterpreterAttribute::TraceSd)
        || self.do_not_trace(call_dag.clone(), Some(sdef))
    {
      return;
    }

    if interpreter.attribute(InterpreterAttribute::TraceBody) {
      println!("{} strategy call", HEADER);
      println!("{}", sdef);
      // call_dags uses the auxiliary symbol we should print it readable
      let call_dag = call_dag.borrow();
      if call_dag.symbol().arity() > 0 {
        println!(
          "call term --> {}({})",
          Token::name(sdef.get_strategy().name()),
          join_iter(call_dag.iter_args(), |_| ", ").collect::<String>()
        );
      }

      if interpreter.attribute(InterpreterAttribute::TraceWhole) {
        println!("subject --> {}", subject.borrow());
      }

      if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
        print_substitution_narrowing(substitution, sdef.variable_info);
      }

    } else {
      if let Some(label) = sdef.name(){
        println!("{}", label);
      } else {
        let strat_id = sdef.get_strategy().name();
        println!("{} (unlabeled definition)", strat_id);
      }
    }
  }

  pub fn trace_pre_sc_application(&self, subject: RcDagNode, sc: Option<&SortConstraint>) {
    let interpreter = self.interpreter.upgrade().unwrap();
    if interpreter.attribute(InterpreterAttribute::Profile) {
      self.root
          .borrow()
          .symbol()
          .get_module()
          .upgrade()
          .unwrap()
          .borrow()
          .profile_mb_rewrite(subject.clone(), sc);
    }
    // if interpreter.attribute(InterpreterAttribute::PrintAttribute) {
    //   check_for_print_attribute(MetadataStore::MEMB_AX, sc);
    // }

    if self.handle_debug(subject.clone(), sc)
        || !self.local_trace_flag
        || !interpreter.attribute(InterpreterAttribute::TraceMb)
        || self.do_not_trace(subject.clone(), sc)
    {
      return;
    }

    if interpreter.attribute(InterpreterAttribute::TraceBody) {
      println!("{} membership axiom", HEADER);
    }

    if let Some(constraint) = sc {
      if interpreter.attribute(InterpreterAttribute::TraceBody) {
        println!("{}", constraint);
        if interpreter.attribute(InterpreterAttribute::TraceSubstitution) {
          print_substitution(&self.substitution, &constraint.variable_info, &NatSet::default());
        }
      } else {
        if let Some(label) = constraint.name(){
          println!("{}", label);
        } else {
          println!("(unlabeled membership axiom)");
        }
      }
    } else {
      println!("(built-in membership axiom for symbol {})", subject.symbol());
    }

    if interpreter.attribute(InterpreterAttribute::TraceWhole) {
      println!("Whole: {}", self.root());
    }

    if interpreter.attribute(InterpreterAttribute::TraceRewrite) {
      if let Some(constraint) = sc {
        println!(
          "{}: {} becomes {}",
          subject.get_sort(),
          subject.borrow(),
          constraint.get_sort()
        );
      }
    }
  }

}
