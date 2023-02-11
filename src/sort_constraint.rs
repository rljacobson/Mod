use std::rc::Rc;

pub type RcSortConstraint = Rc<SortConstraint>;

// Todo: Determine if SortConstraints should be implemented.
pub struct SortConstraint {

}

pub struct SortConstraintTable {
  constraints: Vec<RcSortConstraint>,
  is_complete: bool
}