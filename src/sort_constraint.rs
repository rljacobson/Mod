use std::rc::Rc;

pub type RcSortConstraint = Rc<SortConstraint>;

// Todo: Determine if SortConstraints should be implemented.
pub struct SortConstraint {
  garbage: i32
}

#[derive(Clone)]
pub struct SortConstraintTable {
  constraints: Vec<RcSortConstraint>,
  is_complete: bool
}

impl SortConstraintTable {
  pub fn is_empty(&self) -> bool {
    self.constraints.is_empty()
  }
}
