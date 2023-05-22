/*!

A `Rule` is like an equation except that it is nondeterministic and the system it is part of is not assumed to be confluent.

*/




use pratt::{Channel, log};
use yansi::Paint;

use crate::{
  abstractions::{
    IString,
    NatSet
  },
  core::{
    condition_fragment::{
      Condition,
      repr_condition
    },
    format::{
      FormatStyle,
      Formattable
    },
    pre_equation::{
      impl_pre_equation_formattable,
      PreEquation,
      PreEquationMembers,
    },
    pre_equation_attributes::PreEquationAttribute,
    rewrite_context::RewritingContext,
    RHSBuilder,
    interpreter::rewrite_context::ContextAttribute,
  },
  theory::{
    LHSAutomaton,
    RcDagNode,
    RcLHSAutomaton,
    RcTerm,
  },
  UNDEFINED
};
use crate::core::interpreter::InterpreterAttribute;
use crate::theory::index_variables;

pub struct Rule {
  members: PreEquationMembers,

  // Rule specific members
  rhs_term: RcTerm,
  rhs_builder: RHSBuilder,
  non_extension_lhs_automaton: Option<RcLHSAutomaton>,
  extension_lhs_automaton: Option<RcLHSAutomaton>,

}

impl Rule {

  fn new(name: Option<IString>, lhs_term: RcTerm, rhs_term: RcTerm, condition: Condition) -> Self {
    // assert!(rhs.is_some(), "null rhs");

    let members =
      PreEquationMembers {
        name,
        attributes: Default::default(),
        lhs_term,
        lhs_automaton: None,
        lhs_dag: None,
        condition,
        variable_info: Default::default(),
        index_within_parent_module: UNDEFINED,
        parent_module: Default::default(),
      };

    Rule{
      members,
      rhs_term,
      rhs_builder: Default::default(),
      non_extension_lhs_automaton: None,
      extension_lhs_automaton: None,
    }
  }

  pub fn is_narrowing(&self) -> bool {
    self.members.attributes.has_attribute(PreEquationAttribute::Narrowing)
  }

  fn check(&mut self) {
    let mut bound_variables = PreEquation::check(self);

    self.rhs_term.borrow_mut().normalize(false);
    index_variables(self.rhs_term.clone(), self.variable_info_mut());

    let mut unbound_variables = self.rhs_term.borrow().occurs_below().difference(&bound_variables);
    self.variable_info_mut().add_unbound_variables(&unbound_variables);

    if !self.is_nonexec() && !self.variable_info().unbound_variables.is_empty() {
      let mindex = self.variable_info().unbound_variables.min_value().unwrap();
      let min_variable = self.variable_info().index2variable(mindex);

      let warning = format!(
        "{}: variable {} is used before it is bound in rule:\n{}",
        Paint::magenta(self.repr(FormatStyle::Simple)),
        min_variable.borrow(),
        self.repr(FormatStyle::Default)
      );
      log(Channel::Warning, 1, warning.as_str());

      // Rules with variables used before they are bound have a legitimate purpose - they can be used with metaApply()
      // and a substitution. So we just make the rule nonexec rather than marking it as bad.

      self.set_nonexec();
    }
  }
}

/*
impl Formattable for Rule {
  fn repr(&self, style: FormatStyle) -> String {
    let mut accumulator = String::new();

    if style != FormatStyle::Simple {
      if self.has_condition() {
        accumulator.push('c');
      }
      accumulator.push_str("rl ");
    }

    accumulator.push_str(
      format!(
        "{} => {}",
        self.lhs_term().borrow().repr(style),
        self.rhs_term.borrow().repr(style)
      ).as_str()
    );

    if self.has_condition() {
      accumulator.push(' ');
      repr_condition(self.condition(), style);
    }

    { // Scope of attributes
      let attributes = self.members.attributes;
      if !attributes.is_empty() {
        accumulator.push_str(attributes.repr(style).as_str());
      }
    }

    if style != FormatStyle::Simple {
      accumulator.push_str(" .");
    }

    accumulator
  }
}*/

impl_pre_equation_formattable!(
  Rule,
  "rl ",
  self.lhs_term().borrow(),
  self.rhs_term.borrow(),
  "{} => {}"
);

impl PreEquation for Rule {
  fn members_mut(&mut self) -> &mut PreEquationMembers {
    &mut self.members
  }

  fn members(&self) -> &PreEquationMembers {
    &self.members
  }

  fn trace_begin_trial(&self, subject: RcDagNode, context: &mut RewritingContext) -> Option<i32> {
    context.trace_begin_trial(subject, self, InterpreterAttribute::TraceRl)
  }
}
