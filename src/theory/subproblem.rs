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

use crate::local_bindings::LocalBindings;
use crate::rewrite_context::RewritingContext;
use crate::Substitution;
use crate::theory::LhsAutomaton;

//	These traits must be derived from for equational theories that
//	need to generate matching or unification subproblems or
//	pass back extension information.

pub trait ExtensionInfo {
  // Todo: Implement `ExtensionInfo`.
}

/// Represents a subproblem of a matching problem.
pub trait Subproblem {
  fn solve(&mut self, find_first: bool, context: &mut RewritingContext) -> bool;
}


pub struct VariableAbstractionSubproblem {
  abstracted_pattern  : Box<LhsAutomaton>,
  abstraction_variable: u32,
  variable_count      : u32,
  difference          : Option<LocalBindings>,
  subproblem          : Option<Box<Subproblem>>,
  local               : Substitution,      // Todo: How does this differ from `difference`?
  solved              : bool
}

impl VariableAbstractionSubproblem {
  pub fn new(abstracted_pattern: Box<LhsAutomaton>, abstraction_variable: u32, variable_count: u32) -> Self {
    VariableAbstractionSubproblem {
      abstracted_pattern,
      abstraction_variable,
      variable_count,
      difference: Some(LocalBindings::default()),
      subproblem: None,
      local     : Default::default(),
      solved    : false
    }
  }
}


impl Subproblem for VariableAbstractionSubproblem {
  fn solve(
    &mut self,
    mut find_first: bool,
    context       : &mut RewritingContext
  ) -> bool {
    if find_first {
      self.local.copy(context);

      let v = context.solution.value(self.abstraction_variable);
      assert!(v.is_some(), "Unbound abstraction variable");
      let v = v.unwrap();
      if !self.abstracted_pattern.match_(
        v.clone(),
        &mut self.local,
        self.subproblem.as_deref_mut(),
        None
      )
      {
        return false;
      }

      self.difference = self.local.subtract(&context.solution);
      if Some(difference) = self.difference.as_ref() {
        difference.assert(&context.solution)
      }

      if let Some(subproblem) = &mut self.subproblem {
        if subproblem.solve(true, context) {
          return true;
        }
      } else {
        return true;
      }

    } else {
      if let Some(subproblem) = &mut self.subproblem {
        if subproblem.solve(true, context) {
          return true;
        }
      } else {
        return true;
      }
    }

    if Some(difference) = self.difference.as_ref() {
      difference.retract(&context.solution);
      self.difference = None;
    }

    self.subproblem = None;
    false
  }
}


pub struct SubproblemSequence {
  sequence: Vec<Box<dyn Subproblem>>,
}

impl SubproblemSequence {
  pub fn new() -> Self {
    SubproblemSequence {
      sequence: vec![],
    }
  }

  pub fn add(&mut self, subproblem: Box<dyn Subproblem>) {
    self.sequence.push(subproblem);
  }

  pub fn extract_subproblem(mut self) -> Box<dyn Subproblem> {
    if self.sequence.len() == 1 {
      self.sequence.pop().unwrap()
    } else {
      Box::new(self)
    }
  }

}

impl Subproblem for SubproblemSequence {
  fn solve(&mut self, mut find_first: bool, context: &mut RewritingContext) -> bool {
    let len = self.sequence.len();
    let mut i = match find_first {
      true => 0,
      false => len - 1
    };

    loop {
      find_first = self.sequence[i].solve(find_first, context);
      if find_first {
        i += 1;
        if i == len { break; }
      } else {
        i -= 1;
        if i < 0 { break; }
      }
    }

    return find_first;
  }
}
