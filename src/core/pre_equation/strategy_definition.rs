/*!

UNIMPLEMENTED

*/


use crate::core::pre_equation::impl_pre_equation_formattable;

pub struct StrategyDefinition {

}


impl_pre_equation_formattable!(
  StrategyDefinition,
  "sd ",
  self.lhs_term().borrow(),
  self.rhs_term.borrow(),
  "{} := {}"
);
