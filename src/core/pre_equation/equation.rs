use std::fmt::{Display, Formatter};
use std::rc::Rc;

use crate::{
  abstractions::IString,
  core::{
    RHSBuilder,
    rewrite_context::RewritingContext,
    condition_fragment::{
      Condition,
      repr_condition,
    },
    format::{
      FormatStyle,
      Formattable
    },
    pre_equation::{
      PreEquation,
      PreEquationMembers
    },
    pre_equation_attributes::{
      PreEquationAttribute,
      PreEquationAttributes
    },
    VariableInfo
  },
  NONE,
  theory::{
    RcDagNode,
    RcLHSAutomaton,
    RcTerm
  }
};
use crate::core::interpreter::InterpreterAttribute;


pub type RcEquation = Rc<Equation>;

pub struct Equation {
  members            : PreEquationMembers,
  rhs_term           : RcTerm,
  rhs_builder        : RHSBuilder,
  fast_variable_count: i32,
}

impl Equation {
  pub fn new(
    name     : Option<IString>,
    lhs_term : RcTerm,
    rhs_term : RcTerm,
    otherwise: bool,     // an "owise" term?
    condition: Condition
  ) -> Self
  {
    let attributes: PreEquationAttributes = if otherwise {
      PreEquationAttribute::Otherwise.into()
    } else {
      PreEquationAttributes::default()
    };

    Equation{
      members: PreEquationMembers{
        name,
        attributes,
        lhs_term,
        lhs_automaton: None,
        lhs_dag      : None,
        condition,
        variable_info: VariableInfo::default(),
        index_within_parent_module: NONE,
        parent_module: Default::default(),
      },
      rhs_term,
      rhs_builder        : RHSBuilder::default(),
      fast_variable_count: 0,
    }
  }


  pub fn fast_variable_count(&self) -> i32 {
    self.fast_variable_count
  }
}


impl PreEquation for Equation {
  fn members_mut(&mut self) -> &mut PreEquationMembers {
    &mut self.members
  }

  fn members(&self) -> &PreEquationMembers {
    &self.members
  }

  fn trace_begin_trial(&self, subject: RcDagNode, context: &mut RewritingContext) -> Option<i32> {
    context.trace_begin_trial(subject, self, InterpreterAttribute::TraceEq)
    // context.trace_begin_eq_trial(subject, self)
  }
}


impl Formattable for Equation {
  fn repr(&self, style: FormatStyle) -> String {
    let mut accumulator = String::new();

    if style != FormatStyle::Simple {
      if self.has_condition() {
        accumulator.push('c');
      }
      accumulator.push_str("eq ");
    }

    accumulator.push_str(
      format!(
        "{} = {}",
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
}
