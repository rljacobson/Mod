/*!

A `Rule` is like an equation except that it is nondeterministic and the system it is part of is not assumed to be confluent.

*/

use tiny_logger::{Channel, log};
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
      PreEquationAttribute,
      PreEquationKind,
      Rule
    },
    rewrite_context::{
      ContextAttribute,
      RewritingContext,
    },
    TermBag
  },
  theory::{
    index_variables,
    LHSAutomaton,
    RcDagNode,
    RcLHSAutomaton,
    RcTerm,
    term_compiler::compile_top_rhs
  },
  UNDEFINED,
};


fn new(name: Option<IString>, lhs_term: RcTerm, rhs_term: RcTerm, condition: Condition) -> PreEquation {
  // assert!(rhs.is_some(), "null rhs");
  PreEquation {
    name,
    attributes                : Default::default(),
    lhs_term,
    lhs_automaton             : None,
    lhs_dag                   : None,
    condition,
    variable_info             : Default::default(),
    index_within_parent_module: UNDEFINED,
    parent_module             : Default::default(),
    kind                      : Rule {
        rhs_term,
        rhs_builder                : Default::default(),
        non_extension_lhs_automaton: None,
        extension_lhs_automaton    : None,
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
      let min_variable = this.variable_info.index_to_variable(mindex).unwrap();

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

pub(crate) fn compile(this: &mut PreEquation, compile_lhs: bool) {
  if this.is_compiled() {
    return;
  }
  this.attributes.set(PreEquationAttribute::Compiled);
  let mut available_terms = TermBag::new(); // terms available for reuse

  // Since rules can be applied in non-eager subterms, if we have
  // a condition we must consider all variables to be non-eager
  // to avoid having a condition reduce a lazy subterm.
  this.compile_build(&mut available_terms, !this.has_condition());

  if let Rule {rhs_term, mut rhs_builder, ..} = &mut this.kind {


    // HACK: we pessimize the compilation of unconditional rules to avoid
    // left->right subterm sharing that would break narrowing.
    if !this.has_condition() {
      let mut dummy = TermBag::new();
      compile_top_rhs(
        rhs_term.clone(),
        &mut rhs_builder,
        &mut this.variable_info,
        &mut dummy
      );
    } else {
      compile_top_rhs(
        rhs_term.clone(),
        &mut rhs_builder,
        &mut this.variable_info,
        &mut available_terms
      ); // original code
    }

    this.compile_match(compile_lhs, true);
    rhs_builder.remap_indices(&mut this.variable_info);
  }

  // Make all variables in a rules lhs into condition variables so that
  // if we compile lhs again in get_non_ext_lhs_automaton() or get_ext_lhs_automaton()
  // it will be compiled to generate all matchers rather than just those
  // that differ on variables in the condition.
  this.variable_info.add_condition_variables(this.lhs_term.borrow().occurs_below());
}
