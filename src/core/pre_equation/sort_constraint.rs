/*!

Methods that are specific to sort constraints (membership axioms) can be called like this:

```rust
equation::fast_variable_count(&this);
```

*/

use pratt::{Channel, log};
use yansi::Paint;

use crate::{
  abstractions::{IString, NatSet},
  core::{
    condition_fragment::Condition,
    format::{
      FormatStyle,
      Formattable
    },
    pre_equation::{
      PreEquation,
      PreEquationAttribute,
      SortConstraint
    },
    sort::RcSort
  },
  theory::RcTerm,
  UNDEFINED
};

pub fn new(
  name     : Option<IString>,
  lhs_term : RcTerm,
  sort     : RcSort,
  condition: Condition
) -> PreEquation {

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
    kind: SortConstraint {
      sort
    }
  }
}

pub(crate) fn check(this: &mut PreEquation) {
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

    // No legitimate use for such sort constraints so mark it as bad.
    this.attributes |= PreEquationAttribute::Bad;
  }
}
