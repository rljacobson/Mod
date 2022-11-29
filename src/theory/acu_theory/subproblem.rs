/*!



 */


use crate::rewrite_context::RewritingContext;
use crate::theory::subproblem::Subproblem;

pub struct ACUSubproblem {

}

impl Subproblem for ACUSubproblem {
  fn solve(&mut self, find_first: bool, context: &mut RewritingContext) -> bool {
    false
  }
}
