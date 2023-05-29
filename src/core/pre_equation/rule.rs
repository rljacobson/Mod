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
    interpreter::InterpreterAttribute,
    pre_equation::{
      PreEquation,
      PreEquationAttribute
    },
    rewrite_context::{
      ContextAttribute,
      RewritingContext,
    },
    RHSBuilder,
  },
  theory::{
    index_variables,
    LHSAutomaton,
    RcDagNode,
    RcLHSAutomaton,
    RcTerm,
  },
  UNDEFINED,
};
use crate::core::pre_equation::{PreEquationKind, Rule};


fn new(name: Option<IString>, lhs_term: RcTerm, rhs_term: RcTerm, condition: Condition) -> PreEquation {
  // assert!(rhs.is_some(), "null rhs");
  PreEquation {
    name,
    attributes: Default::default(),
    lhs_term,
    lhs_automaton: None,
    lhs_dag: None,
    condition,
    variable_info: Default::default(),
    index_within_parent_module: UNDEFINED,
    parent_module: Default::default(),
    kind: Rule {
      rhs_term,
      rhs_builder: Default::default(),
      non_extension_lhs_automaton: None,
      extension_lhs_automaton: None,
    }
  }
}

pub(crate) fn check(this: &mut PreEquation, bound_variables: NatSet) {
  if let Rule{
    rhs_term,
    rhs_builder,
    non_extension_lhs_automaton,
    extension_lhs_automaton
  } = &this.kind
  {
    rhs_term.borrow_mut().normalize(false);
    index_variables(rhs_term.clone(), &mut this.variable_info);

    let mut unbound_variables = rhs_term.borrow().occurs_below().difference(&bound_variables);
    this.variable_info.add_unbound_variables(&unbound_variables);

    if !this.is_nonexec() && !this.variable_info.unbound_variables.is_empty() {
      let mindex = this.variable_info.unbound_variables.min_value().unwrap();
      let min_variable = this.variable_info.index2variable(mindex).unwrap();

      let warning = format!(
        "{}: variable {} is used before it is bound in {}:\n{}",
        Paint::magenta(this.repr(FormatStyle::Simple)),
        min_variable.borrow(),
        this.kind.noun(),
        this.repr(FormatStyle::Default)
      );
      log(Channel::Warning, 1, warning.as_str());

      // Rules with variables used before they are bound have a legitimate purpose - they can be used with metaApply()
      // and a substitution. So we just make the rule nonexec rather than marking it as bad.

      this.set_nonexec();
    }
  } else {
    unreachable!()
  }
}
