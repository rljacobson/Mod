/*!

The automaton for the pattern (the LHS).

*/

use std::cell::Cell;
use crate::{
  Substitution,
  theory::{
    acu_theory::{
      ACUDagNode,
      red_black_tree::RedBlackTree,
    },
    DagNode,
    ExtensionInfo,
    LhsAutomaton,
    Subproblem,
    Symbol,
    Term
  },
};
use crate::theory::acu_theory::red_black_tree::RedBlackNode;
use crate::theory::Outcome;
use crate::theory::Outcome::Success;
use crate::theory::term::OrderingValue;

use super::{
  automaton_structs::{GroundAlien, NonGroundAlien, TopVariable, MatchStrategy},
  dag_node::ACUArguments
};


pub struct ACULHSAutomaton<'a> {
  max_pattern_multiplicity: u32,
  current_multiplicity    : Vec<u32>,
  total_lower_bound       : u32,
  total_upper_bound       : u32,
  total_multiplicity      : u32,
  total_non_ground_aliens_multiplicity: u32,
  last_unbound_variable   : u32,
  unbound_variable_count  : u32,
  ground_aliens           : Vec<GroundAlien<'a>>,
  grounded_out_aliens     : Vec<NonGroundAlien<'a>>,
  non_ground_aliens       : Vec<NonGroundAlien<'a>>,
  current                 : RedBlackTree,
  top_symbol              : Box<Symbol>,
  top_variables           : Vec<TopVariable<'a>>,

  collapse_possible: bool,
  match_at_top     : bool,
  tree_match_ok    : bool,
  match_strategy   : MatchStrategy,

  matched_multiplicity: u32,
}

impl ACULHSAutomaton {

  pub fn match_(
    &mut self,
    subject       : &mut dyn DagNode,
    solution      : &mut Substitution,
    mut returned_subproblem: Option<&mut dyn Subproblem>,
    mut extension_info: Option<&mut dyn ExtensionInfo>,
  ) -> bool
  {
    if subject.symbol() != self.top_symbol() {
      if self.collapse_possible {
        return self.collapse_match(subject, solution, returned_subproblem, extension_info)
      }
      return false;
    }

    assert_eq!(self.match_at_top, extension_info.is_some(), "match_at_top disagreement");

    // todo: What is the point of this?
    // returned_subproblem  = 0;

    if let Some(s) = subject.as_any().downcast_mut::<ACUDagNode>() {
      //	Check to see if we should use red-black matcher.
      if let ACUArguments::Tree(t) = &s.args {
        if self.tree_match_ok {
          let r = self.tree_match(t, solution, &mut returned_subproblem, &mut extension_info);
          // r == true || r == false
          match r {
            Outcome::Falures => { return false; }
            Outcome::Success => { return true;  }
            _ => { /* pass */ }
          }
        }
        // Convert from red-black tree representation to a vector representation.
        s.to_list_arguments();
      }
      // From here on we can assume that subject s has a vector representation.

      //	Check for trivial failure and eliminate stuff that can only
      //	be matched in one way.
      if !self.multiplicity_checks(s) ||
          !self.eliminate_ground_aliens(s) ||
          !self.eliminate_bound_variables_for_subject(s, solution) ||
          !self.eliminate_grounded_out_aliens_for_subject(s, solution) {
        return false;
      }

      if extension_info.is_none() {
        //	Matching without extension special cases:
        if self.last_unbound_variable == -1 /* NONE */ {
          if self.total_non_ground_aliens_multiplicity != self.compute_total_multiplicity() {
            return false;
          }
          if self.total_non_ground_aliens_multiplicity == 0 {
            return true;
          }
          if self.match_strategy == MatchStrategy::AliensOnly {
            return self.aliens_only_match(s, solution, returned_subproblem);
          }
        }
      }


    } else {
      panic!("ACULHSAutomaton::match  called with non ACU DagNode. This is a bug.");
    }


    false
  }

  fn collapse_match(
    &mut self, subject: &dyn DagNode,
    solution: &mut Substitution,
    returned_subproblem: Option<&mut dyn Subproblem>,
    extension_info: Option<&mut dyn ExtensionInfo>
  ) -> bool
  {
    false
  }

  fn multiplicity_checks(&mut self, subject: &ACUDagNode ) -> bool {
    // Copy argument multiplicities and check for trivial failure.
    if self.max_pattern_multiplicity  > 1 {
      //	Because failure here is common we check this first.
      let mut ok = false;
      for (_, multiplicity) in subject.iter() {
        if multiplicity >= self.max_pattern_multiplicity {
          ok = true;
          break;
        }
      }
      if !ok {
        return false;
      }
    }

    // ok:
    self.current_multiplicity.resize(subject.len(), 0);
    let mut total_subject_multiplicity = 0;
    for (idx, (_, multiplicity)) in subject.iter().enumerate() {
      self.current_multiplicity[idx] = multiplicity;
      total_subject_multiplicity += multiplicity;

    }

    if total_subject_multiplicity < self.total_lower_bound ||
        total_subject_multiplicity > self.total_upper_bound {
      return false;
    }

    self.total_multiplicity = total_subject_multiplicity;
    true
  }

  /// There are two versions of this method, one that takes a subject and one that doesn't.
  fn eliminate_ground_aliens(&mut self, subject: &ACUDagNode) -> bool {
    for alien in self.ground_aliens.iter() {
      // todo: This check is not in Maude.
      // if self.current_multiplicity.is_empty() {
      //   return false;
      // }
      let pos = subject.binary_search_by_term(alien.term.as_ref());
      if pos < 0 {
        return false;
      }
      self.current_multiplicity[pos] -= alien.multiplicity;
      if self.current_multiplicity[pos] < 0 {
        return false;
      }
    }
    true
  }


  fn eliminate_bound_variables_for_subject(
    &mut self,
    subject: &mut ACUDagNode,
    solution: &mut Substitution
  ) -> bool
  {
    let top_variables_count = self.top_variables.len();
    self.last_unbound_variable = -1; // NONE
    for i in 0..top_variables_count {
      if let Some(d) = solution.get_mut(self.top_variables[i].index as usize){
        if !(
              subject.eliminate_subject(
                d,
                self.top_variables[i].multiplicity,
                &self.current_multiplicity
              )
            )
        {
          return false;
        }
      } else {
        //	Make linked list of unbound variables.
        self.top_variables[i].previous_unbound = self.last_unbound_variable;
        self.last_unbound_variable = i as u32;
      }

    } // end for loop

    return true;
  }

  fn eliminate_grounded_out_aliens_for_current(&mut self, solution: &mut Substitution) -> bool {
    'next_alien:
    for alien in self.grounded_out_aliens.iter_mut() {
      assert!(alien.term.is_some(), "shouldn't be running on unstable terms");
      if self.current.size != 0 {
        if let Some(mut j) = self.current.find_first_potential_match(alien.term.unwrap(), solution) {
          let mut a = alien.lhs_automaton.as_mut();
          let mut d_rb_node = Cell::<RedBlackNode>::get_mut((&j).get().as_deref_mut().unwrap());

          while !j.is_null() {
            if a.match_(d_rb_node.dag_node.as_ref(), solution, None){
              let mut multiplicity = alien.multiplicity;

              if d_rb_node.multiplicity < multiplicity {
                return false;
              }

              self.current.delete_multiplicity_at_cursor(&mut j, multiplicity);
              self.matched_multiplicity += multiplicity;
              continue 'next_alien;
            }

            j.move_next();
            if !j.valid() {
              break;
            }
            d_rb_node = Cell::<RedBlackNode>::get_mut((&j).get().as_deref_mut().unwrap());
            if  t.partial_compare(solution, d) == OrderingValue::Less {
              //	Since t is less then d, it will also be less than
              //	all next nodes so we can quit now.
              break
            }
          }

        }
      }
      return false;
    }

    true
  }

  fn eliminate_grounded_out_aliens_for_subject(
    &mut self,
    subject : &mut ACUDagNode,
    solution: &mut Substitution
  ) -> bool
  {
    // The args of subject should always be a vector.
    if let ACUArguments::List(args) = &subject.args {
      let arg_count = args.len();

      'next_alien:
      for i in self.grounded_out_aliens.iter() {
        let mut j: usize = match i.term {
          Some(t) => subject.find_first_potential_match(t, solution) as usize,
          None => {
            0usize
          }
        };

        if j < arg_count {
          let a    : &dyn LhsAutomaton = i.lhs_automaton.as_ref();
          let mut d: &dyn DagNode      = args[j].dag_node.as_ref();

          loop {
            if a.match_(d, solution, None) {
              self.current_multiplicity[j] -= i.multiplicity;
              if self.current_multiplicity[j] < 0 {
                return false;
              }

              continue 'next_alien;
            } // end successful a.match_

            j += 1;
            if j == arg_count {
              break;
            }
            d = args[j].dag_node.as_ref();
            if i.term.is_some() && i.term.unwrap().partial_compare(solution, d) == OrderingValue::Less {
              break;
            }
          } // end loop
        } // end if j < arg_count
        return false;
      }
    }

    true
  }


  /// Implementation for AC/ACU matcher that works on red-black trees.
  fn eliminate_bound_variables(&mut self, solution: &mut Substitution) -> Outcome {
    self.unbound_variable_count = 0;
    for i in self.top_variables {
      if let Some(d) = solution.value(i.index){
        if d.symbol() == self.top_symbol {
          return Outcome::Undecided /* UNDECIDED */;
        }

        // todo: implement get_identity on Symbol
        match self.top_symbol.get_identity() {
          | None
          | Some(identity) if !identity.equal(d) => {
            if self.current.size == 0 {
              return Outcome::Failure /* false */;
            }
            // todo: This is wrong since we changed node_for_key() to return a cursor instead of a dagnode.
            // todo: Does node_for_key take a DagNode or Term?
            if let Some(mut j) = self.current.find_mut(d) {
              let multiplicity = i.multiplicity;

              if j.get().unwrap().multiplicity < multiplicity {
                return Outcome::Failure /* false */;
              }

              self.current.delete_multiplicity_at_cursor(&mut j, multiplicity);
              // Todo: Should this go into `delete_multiplicity_at_cursor()` ?
              self.matched_multiplicity += multiplicity;
            } else {
              return Outcome::Failure /* false */;
            }

          }

          _ => {
            /* pass */
          }
        } // end match top_symbol identity


      }//end if i.index is in solution.
      else {
        self.unbound_variable_count += 1;
      }

    } // end for i in self.top_variables
    return Outcome::Success /* true */;
  }

  fn tree_match(
    &mut self,
    subject: &RedBlackTree,
    solution: &mut Substitution,
    returned_subproblem: &mut Option<&mut dyn Subproblem>,
    extension_info: &mut Option<&mut dyn ExtensionInfo>
  ) -> Outcome
  {

    // todo: Is this deep copy necessary? If so, can we check
    //         current.max_multiplicity < self.max_pattern_multiplicity
    //       before the copy?
    let current = subject.clone(); // Deep copy.
    if current.max_multiplicity < self.max_pattern_multiplicity {
      return Outcome::Failure /* false */;
    }

    //	Eliminate subpatterns that must match a specific subterm
    //	in the subject.
    self.matched_multiplicity = 0;
    let r = self.eliminate_bound_variables(solution);
    if r != Outcome::Success /* r != true */ {
      return r;
    }
    if !self.eliminate_ground_aliens_from_current()
        || !self.eliminate_grounded_out_aliens_for_current(solution)
    {
      return Outcome::Failure /* false */;
    }
    if extension_info.is_some()
        && self.unbound_variable_count == 1
        && self.non_ground_aliens.is_empty()
    {
      //	Forced lone variable case.
      for i in self.top_variables {
        if solution.value(i.index) == 0 {
          // todo: implement forced lone variable case.
          return self.forced_lone_variable_case(subject, i, solution, returned_subproblem);
        }
      }
      panic!("didn't find unbound variable");
    }

    if self.match_strategy == MatchStrategy::Full {
      //	We're here because treeMatchOK was true, which implies:
      //	  We're not matching at the top
      //	  Expected nrUnboundVariables = 1
      //	  That one variable has unbounded sort and multiplicity 1
      //	  Number of NGAs = 1
      //	  That one NGA is stable and has multiplicity 1
      //
      assert!(nrUnboundVariables <= 1, "nrUnboundVariables = {}", self.unbound_variable_count);
    }
    Outcome::Failure
  }

}

