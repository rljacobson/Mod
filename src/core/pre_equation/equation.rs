/*!

Methods that are specific to equations can be called like this:

```rust
equation::fast_variable_count(&this);
```

*/

use std::{
  fmt::{Display, Formatter},
  rc::Rc
};

use tiny_logger::{Channel, log};
use yansi::Paint;

use crate::{
  abstractions::{IString, NatSet},
  core::{
    condition_fragment::{
      Condition,
      repr_condition,
    },
    format::{
      FormatStyle,
      Formattable
    },
    interpreter::InterpreterAttribute,
    pre_equation::{
      Equation,
      PreEquation,
      PreEquationAttribute,
      PreEquationAttributes,
      PreEquationKind,
      sort_constraint,
      sort_constraint_table::SortConstraintTable,
    },
    rewrite_context::RewritingContext,
    VariableInfo,
  },
  NONE,
  theory::{
    RcDagNode,
    RcLHSAutomaton,
    RcTerm,
    term_compiler::compile_top_rhs,
  },
};
use crate::core::automata::RHSBuilder;
use crate::core::rewrite_context::ContextAttribute;
use crate::core::TermBag;
use crate::theory::index_variables;


pub fn new(
  name     : Option<IString>,
  lhs_term : RcTerm,
  rhs_term : RcTerm,
  otherwise: bool,     // an "owise" term?
  condition: Condition
) -> PreEquation
{
  let attributes: PreEquationAttributes = if otherwise {
    PreEquationAttribute::Otherwise.into()
  } else {
    PreEquationAttributes::default()
  };

  PreEquation{
    name,
    attributes,
    lhs_term,
    lhs_automaton: None,
    lhs_dag      : None,
    condition,
    variable_info: VariableInfo::default(),
    parent_module: Default::default(),
    index_within_parent_module: NONE,

    kind: Equation {
      rhs_term,
      rhs_builder        : RHSBuilder::default(),
      fast_variable_count: 0,
    }
  }
}

pub(crate) fn check(this: &mut PreEquation, bound_variables: NatSet) {
  if let Equation {rhs_term, .. } = &this.kind {
    {
      let mut rhs_term = rhs_term.borrow_mut();
      rhs_term.normalize(false);
    }
    index_variables(rhs_term.clone(), &mut this.variable_info);

    let mut unbound_variables = rhs_term.borrow_mut().occurs_below_mut();
    unbound_variables.difference_in_place(&bound_variables);
    this.variable_info.add_unbound_variables(unbound_variables);

    // The remainder just happens to be identical to the check for sort constraints.
    sort_constraint::check(this);

  }
}

pub(crate) fn compile(this: &mut PreEquation, mut compile_lhs: bool) {
  if this.is_compiled() {
    return;
  }
  this.attributes.set(PreEquationAttribute::Compiled);

  let mut available_terms = TermBag::new();  // terms available for reuse
  this.compile_build(&mut available_terms, true);

  // Destructure
  if let Equation {rhs_term, rhs_builder, fast_variable_count} = &mut this.kind {

    if this.is_variant() {
      //
      // If the equation has the variant attribute, we disallow left->right sharing so
      // that the rhs can still be instantiated, even if the substitution was made by
      // unification.
      //
      let mut dummy = TermBag::new();
      compile_top_rhs(rhs_term.clone(), rhs_builder, &mut this.variable_info, &mut dummy);
      //
      // For an equation with the variant attribute we always compile the lhs, even if the parent symbol
      // doesn't make use of the compiled lhs (in the free theory because it uses a discrimination
      // net for lhs matching).
      //
      compile_lhs = true;
    }
    else {
      compile_top_rhs(rhs_term.clone(), rhs_builder, &mut this.variable_info, &mut available_terms);  // normal case
    }

    this.compile_match(compile_lhs, true);
    rhs_builder.remap_indices(&mut this.variable_info);
    *fast_variable_count = if this.has_condition() {
      NONE
    }
    else {
      this.variable_info.protected_variable_count()
    };  // HACK

  } else {
    unreachable!("Tried to compile nonequation as an equation. This is a bug.")
  }
}
