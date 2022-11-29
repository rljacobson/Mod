/*!

A `Subproblem` corresponds roughly to the `MatchGenerator`s of Loris.

# Eker96

## Matching Phase

Here the result of a match is a partial substitution which contains variables that can easily be
determined to have the same value in all matching substitutions; together with a subproblem object
which is a compact representation of the possible values for the variables not mentioned in the
partial substitution. For a simple pattern the partial substitution might contain bindings for all
the variables in the pattern in which case the empty subproblem object denoted by $\emptyset$ is returned.
Of course the matching phase could fail altogether in which case the pair (fail, $\emptyset$) is returned.

## Subproblem Solving Phase

For many simple patterns this phase will be unnecessary as the matching phase will have uniquely
bound all the variables. For more complex patterns we are left with a partial substitution
and a subproblem object which may contain nested subproblem sub-objects. In the subproblem
solving phase the subproblem object is searched for consistent sets of solutions to the unbound
variables; each such set corresponds to a different solution to the original matching problem.

For implementation purposes subproblem objects actually contain state information to record which
possibilities have already been tried and the returned subproblem object is really the original subproblem
object with its state updated. Thus, solutions can be extracted from the subproblem object as needed.

*/

use crate::rewrite_context::RewritingContext;

//	These traits must be derived from for equational theories that
//	need to generate matching or unification subproblems or
//	pass back extension information.

pub trait ExtensionInfo {}

/// Represents a subproblem of a matching problem. The `delayed_solve` features have trivial default implementations
/// and so are optionally implemented.
pub trait Subproblem {
  fn solve(&mut self, find_first: bool, context: &mut RewritingContext) -> bool;

  // region Delayed Subproblem

  fn delayed_solve(&mut self, _find_first: bool, _context: &mut RewritingContext) -> bool {
    return false;
  }

  fn is_solved(&self) -> bool{
    return false;
  }

  fn set_solved(&mut self, _solved: bool) {  }

  // endregion

}

