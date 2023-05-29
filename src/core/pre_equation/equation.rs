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

use pratt::{Channel, log};
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
    },
    rewrite_context::RewritingContext,
    RHSBuilder,
    VariableInfo,
  },
  NONE,
  theory::{
    RcDagNode,
    RcLHSAutomaton,
    RcTerm
  },
};
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

    /*if !this.is_nonexec() && !this.variable_info.unbound_variables.is_empty() {
      let mindex = this.variable_info.unbound_variables.min_value().unwrap();
      let min_variable = this.variable_info.index2variable(mindex);

      let warning = format!(
        "{}: variable {} is used before it is bound in {}:\n{}",
        Paint::magenta(this.repr(FormatStyle::Simple)),
        min_variable.borrow(),
        this.kind.noun(),
        this.repr(FormatStyle::Default)
      );
      log(Channel::Warning, 1, warning.as_str());

      // No legitimate use for such equations, so mark it as bad.
      this.attributes |= PreEquationAttribute::Bad;
    }*/

  }
}
