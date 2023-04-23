use std::rc::Rc;

pub type RcSortConstraint = Rc<SortConstraint>;


// Todo: Determine if SortConstraints should be implemented.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SortConstraint {
  stub: i32 // TODO: Replace this stub.
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SortConstraintTable {
  constraints: Vec<RcSortConstraint>,
  is_complete: bool
}

impl SortConstraintTable {
  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.constraints.is_empty()
  }
}
