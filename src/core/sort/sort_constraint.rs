use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::abstractions::IString;
use crate::core::VariableInfo;

pub type RcSortConstraint = Rc<SortConstraint>;


// ToDo: Determine if SortConstraints should be implemented.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct SortConstraint {
  name: Option<IString>,
  pub(crate) variable_info: VariableInfo,

}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct SortConstraintTable {
  constraints: Vec<RcSortConstraint>,
  is_complete: bool
}

impl Display for SortConstraint{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "sort_constraint<{:?}>", self.name)
  }
}

impl SortConstraintTable {
  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.constraints.is_empty()
  }
}
