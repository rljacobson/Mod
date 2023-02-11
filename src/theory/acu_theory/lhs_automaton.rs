/*!

The automaton for the pattern (the LHS).

*/

use std::borrow::Borrow;
use std::cell::Cell;
use std::ops::{DerefMut, Sub};
use std::rc::Rc;
use reffers::rc1::Strong;

use crate::{Substitution, theory::{
  Outcome,
  acu_theory::{},
  DagNode,
  ExtensionInfo,
  LhsAutomaton,
  Subproblem,
  Symbol,
  Term,
  dag_node::RcDagNode,
  subproblem::{
    SubproblemSequence,
    VariableAbstractionSubproblem
  },
  symbol::BinarySymbol
}, OrderingValue, Sort};
use crate::sort::{index_leq_sort, SpecialSorts};
use crate::theory::acu_theory::dag_node::NormalizationStatus;
use crate::theory::acu_theory::subproblem::MaybeSubproblem;
use crate::theory::DagPair;

use super::{
  automaton_structs::{
    GroundAlien,
    NonGroundAlien,
    TopVariable,
    MatchStrategy
  },
  dag_node::ACUArguments,
  ACUDagNode,
  red_black_tree::{
    RedBlackTree,
    RcRedBlackTree,
    RedBlackNode
  },
  subproblem::ACULazySubproblem,
  symbol::ACUSymbol
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
  independent_aliens_count: u32,
  ground_aliens           : Vec<GroundAlien<'a>>,
  grounded_out_aliens     : Vec<NonGroundAlien<'a>>,
  non_ground_aliens       : Vec<NonGroundAlien<'a>>,
  current                 : RcRedBlackTree,
  top_symbol              : Box<ACUSymbol>,
  top_variables           : Vec<TopVariable>,

  unique_collapse_automaton: Option<Box<dyn LhsAutomaton>>,
  collapse_possible: bool,
  match_at_top     : bool,
  tree_match_ok    : bool,
  match_strategy   : MatchStrategy,

  // Mutable workspace
  local  : Substitution,
  scratch: Substitution,

  matched_multiplicity: u32,
}

impl ACULHSAutomaton {


  fn collapse_match(
    &mut self,
    subject       : RcDagNode,
    solution      : &mut Substitution,
    extension_info: Option<&mut dyn ExtensionInfo>
  ) -> (bool, MaybeSubproblem)
  {
    if self.unique_collapse_automaton.is_some() {
      return self.unique_collapse_match(subject, solution, extension_info);
    }

    let (outcome, maybe_subproblem) = self.multiway_collapse_match(subject, solution, extension_info);
    if !outcome {
      return (false, None);
    }

    let mut subproblems: SubproblemSequence = SubproblemSequence::new();
    if let Some(subproblem) = maybe_subproblem {
      subproblems.push(subproblem);
    }

    for top_variable in self.top_variables {
      if let Some(abstracted_pattern) = top_variable.abstracted {
        subproblems.add(
          Box::new(
            VariableAbstractionSubproblem::new(
              abstracted_pattern,
              top_variable.index,
              solution.fragile_binding_count() // variable_count
            )
          )
        )
      }
    }

    (true, Some(subproblems.extract_subproblem()))
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
      // Todo: This check is not in Maude.
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


  /// The version of this method that works on trees. Returns the outcome with an optional subproblem.
  fn eliminate_bound_variables_for_current(&mut self, solution: &mut Substitution) -> Outcome {
    self.unbound_variable_count = 0;
    for i in self.top_variables {
      if let Some(d) = solution.value(i.index){
        if d.get_ref().symbol() == self.top_symbol {
          return Outcome::Undecided /* UNDECIDED */;
        }

        match self.top_symbol.get_identity() {
          | None
          | Some(identity) if !identity.get_ref().eq(&d.get_ref()) => {
            if self.current.size == 0 {
              return Outcome::Failure /* false */;
            }

            if let Some(mut j) = self.current.find_term_mut(d.as_ref()) {
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
            if a.match_(d_rb_node.dag_node.as_ref(), solution.deref_mut(), None){
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
            d_rb_node = (&j).get().as_deref_mut().unwrap();
            if alien.term
                    .unwrap()
                    .partial_compare(
                      solution.deref_mut(),
                      d_rb_node.dag_node.as_ref()
                    ) == OrderingValue::Less
            {
              //	Since t is less then d, it will also be less than
              //	all next nodes so we can quit now.
              break;
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
          let a    : &dyn LhsAutomaton       = i.lhs_automaton.as_ref();
          let mut d: Strong<dyn DagNode> = args[j].dag_node.clone();

          loop {
            if a.match_(d.clone(), solution, None) {
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
            d = args[j].dag_node.clone();
            if i.term.is_some()
                && i.term
                    .unwrap()
                    .partial_compare(
                      solution,
                      d.get_ref().as_ref()
                    ) == OrderingValue::Less
            {
              break;
            }
          } // end loop
        } // end if j < arg_count
        return false;
      }
    }

    true
  }


  /// Implementation for AC/ACU matcher that works on red-black trees. Returns the outcome with an optional subproblem.
  fn tree_match(
    &mut self,
    subject: &RedBlackTree,
    solution: &mut Substitution,
    // Todo: What should the type of `extension_info` be?
    extension_info: &mut Option<&mut dyn ExtensionInfo>
  ) -> (Outcome, MaybeSubproblem)
  {

    if subject.max_multiplicity < self.max_pattern_multiplicity {
      return (Outcome::Failure, None) /* false */;
    }
    // Todo: Is this deep copy necessary? If so, can we check
    //         current.max_multiplicity < self.max_pattern_multiplicity
    //       before the copy?
    let mut current = subject.clone(); // Deep copy.

    //	Eliminate subpatterns that must match a specific subterm
    //	in the subject.
    self.matched_multiplicity = 0;
    let r = self.eliminate_bound_variables_for_current(solution);
    if r != Outcome::Success /* r != true */ {
      return (r, None);
    }
    if !self.eliminate_ground_aliens_from_current()
        || !self.eliminate_grounded_out_aliens_for_current(solution)
    {
      return (Outcome::Failure, None) /* false */;
    }
    if extension_info.is_some()
        && self.unbound_variable_count == 1
        && self.non_ground_aliens.is_empty()
    {
      //	Forced lone variable case.
      for i in self.top_variables {
        if solution.value(i.index) == 0 {
          return self.forced_lone_variable_case(subject.is_reduced(), &i, solution);
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
      assert!(self.unbound_variable_count <= 1, "self.unbound_variable_count = {}", self.unbound_variable_count);

      if self.unbound_variable_count != 1 {
        //	The variable we expected to be unbound and act as a collector
        //	variable was bound after all. We could potentially be
        //	smarter here but this is a very unlikely case.
        return (Outcome::Undecided, None);
      }

      if current.size == 0 {
        //	Subject exhausted - we don't expect this to happen in the
        //	red-black case where the subject is expected to be large.
        //	Though we could handle this efficiently it might be tricky
        //	to reach this code, even for test purposes so we don't bother.
        return (Outcome::Undecided, None);
      }

      if current.size == 1 && current.max_multiplicity() == 1 {
        //	Subject reduced to a single item; again it would be tricky
        //	to reach this case, so we don't both with an efficient
        //	implementation.
        return (Outcome::Undecided, None);
      }

      //	The only way we can be here is if we have a nonground alien
      //	and a collector variable, and no extension.
      assert!(self.non_ground_aliens.length() == 1,
             "wrong number of self.non_ground_aliens.length(): {}",
              self.non_ground_aliens.length());
      assert!(extension_info.is_none(), "should not have extension");

      for i in self.top_variables {
        if solution.value(i.index).is_none() {
          assert_eq!(i.multiplicity, 1, "collector multiplicity = {}", i.multiplicity);
          let returned_subproblem = Some(Box::new(
            ACULazySubproblem {
              subject,
              current      : &mut current,
              solution,
              lhs_automaton: &mut self.non_ground_aliens[0].lhs_automaton,
              term         : self.non_ground_aliens[0].term,
              index        : i.index,
              sort         : i.sort
            }
          ));

          return (Outcome::Success, returned_subproblem);
        }
      }

      panic!("didn't find unbound variable");
    }

    //	Match everything else using greedy algorithms.
    // Todo: implement greedy_match
    return self.greedy_match(subject, solution,  extension_info);
  }


  /// The tree version of this method.
  fn forced_lone_variable_case(
    &mut self,
    subject_is_reduced: bool,
    tv: &TopVariable,
    solution: &mut Substitution
  ) -> (Outcome, MaybeSubproblem)
  {
    // Special case: assign identity.
    if self.current.size == 0 {
      if tv.take_identity {
        // Todo: Justify the unwrap.
        solution.bind(tv.index, self.top_symbol.get_identity_dag().unwrap());
        return (Outcome::Success, None);
      }

      return (Outcome::Failure, None);
    }

    let multiplicity = tv.multiplicity;

    //	Special case: assign one subject.
    if self.current.size == 1 && self.current.get_sole_multiplicity() == multiplicity {
      let d = self.current.get_sole_dag_node();
      if d.get_ref().leq(tv.sort.get_ref()) {
        solution.bind(tv.index, d);
        return (Outcome::Success, None);
      }
      return (Outcome::Failure, None);
    }

    //	General case: need to assign everything.
    let mut b_args = // the value of the following if
    if multiplicity == 1 {
      ACUArguments::Tree(self.current.clone())
    } else {
      let mut v: Vec<DagPair> = Vec::new();
      for (dag_node, m) in self.current.iter() {
        if m % multiplicity !=  0  {
          return (Outcome::Failure, None);
        }
        v.push(
          DagPair{
            dag_node,
            multiplicity: m/multiplicity,
          }
        )
      }

      ACUArguments::List(v)
    };

    let b = Strong::new(
        ACUDagNode{
          top_symbol: self.symbol(),
          args: b_args,
          sort: Sort::default(),
          is_reduced: false,
          sort_index: SpecialSorts::SortUnknown as i32,
          normalization_status: NormalizationStatus::Assignment
        }
    );

    if let (true, subproblem) = b.check_sort(&tv.sort) {
      solution.bind(tv.index, b.clone());
      if subject_is_reduced && b.get_sort() != SpecialSorts::SortUnknown {
        b.is_reduced = true;
      }
      return (Outcome::Success, subproblem);
    }

    (Outcome::Failure, subproblem)
  }


  fn greedy_match(
    &mut self,
    subject: &dyn DagNode,
    solution: &mut Substitution,
    extension_info: &mut Option<&mut dyn ExtensionInfo>
  ) -> (Outcome, MaybeSubproblem)
  {
    self.local   = solution.clone(); // greedy matching is speculative so make a copy
    self.scratch = solution.clone(); // keep a scratch copy as well

    'nextNonGroundAlien:
    for (i_idx, i) in self.non_ground_aliens.iter_mut().enumerate() {
      assert!(i.term.is_some(), "shouldn't be running on unstable terms");
      let t: &dyn Term = i.term.unwrap();

      if self.current.size() != 0 {
        if let Some(mut path) = self.current.find_first_potential_match(t, solution) {
          let multiplicity = i.multiplicity;
          let a = &mut i.lhs_automaton;
          let mut j = path.get().unwrap();
          let mut d = j.dag_node;

          loop {
            if j.multiplicity >= multiplicity {
              let (matched, sp) = a.match_(d, &mut self.scratch, None);
              if matched {
                if Some(sp) = sp {
                  return (Outcome::Undecided, None);
                }

                self.local = scratch.clone();
                ;  // preserve any new bindings
                self.current.delete_multiplicity_at_cursor(path, multiplicity);
                self.matched_multiplicity += multiplicity;
                continue 'nextNonGroundAlien;
              }
              self.local = self.scratch.clone();  // restore scratch copy
            }
            let next = path.next();
            if next.is_none() {
              break;
            }
            j = next.unwrap();
            d = j.dag_node;
            if t.partial_compare(solution, d) == OrderingValue::Less {
              //	Since t is less then d, it will also be less than
              //	all next nodes so we can quit now.
              break;
            }
          }

        }
      }

      return if i_idx < self.independent_aliens_count as usize
      {
        (Outcome::Failure, None)
      } else {
        (Outcome::Undecided, None)
      }
    }

    if self.greedy_pure_match(subject, &mut self.local, extension_info) {
      // Todo: Can I do this instead of copy?
      std::mem::swap(solution, &mut self.local);
      return (Outcome::Success, None);
    }

    // When the pure matching step fails we always treat it as UNDECIDED for safety.
    (Outcome::Undecided, None)
  }


  /// Tree version of greedy_pure_match
  pub fn greedy_pure_match(
    &mut self,
    subject: &dyn DagNode,
    solution: &mut Substitution,
    extension_info:  &mut Option<&mut dyn ExtensionInfo>
  ) -> Outcome {
    //	Greedy pure matching can fail for so many reasons
    //	in the red-black case, we don't bother trying to
    //	detect true failure: false always means UNDECIDED.

    for i in &self.top_variables {
      if solution.value(i.index) == 0 {
        self.unbound_variable_count -= 1;
        if self.current.size == 0 {

          if !(i.take_identity) {
            return Outcome::Failure;
          }
          solution.bind(i.index, self.top_symbol.get_identity_dag().unwrap());
          if self.unbound_variable_count == 0 {
            break;
          }

        } else {
          if self.unbound_variable_count == 0 {
            // Implement `try_to_bind_last_variable()`
            if !self.try_to_bind_last_variable(subject, i, solution) {
              return Outcome::Failure;
            }
            break;
          } else {
            // Implement `try_to_bind_variable()`
            if !self.try_to_bind_variable(i, solution) {
              return Outcome::Failure;;
            }
          }
        }
      }
    }

    if self.current.size == 0 {
      //	Everything matched; fill out empty extension if needed.
      if let Some(extension_info) = *extension_info {
        extension_info.set_valid_after_match(true);
        extension_info.set_matched_whole(true);
      }
    } else {
      //	Stuff left over; see if we can put it in the extension.
      if let Some(extension_info) = *extension_info {
        if self.matched_multiplicity >= 2 {
          extension_info.set_valid_after_match(true);
          extension_info.set_matched_whole(false);
          if self.current.size == 1 && self.current.max_multiplicity() == 1 {
            extension_info.set_unmatched(self.current.get_sole_dag_node());
          } else {
            // ToDo: What should be done about this tree node creation? Is it creating a new tree?
            extension_info.set_unmatched(ACUDagNode::new(&self.top_symbol, self.current));
          }
        } else {
          return Outcome::Failure;
        }
      } else {
        return Outcome::Failure;
      }
    }

    return Outcome::Success;
  }

  fn try_to_bind_last_variable(
    &mut self,
    subject: &ACUDagNode,
    top_variable: &TopVariable,
    solution: &mut Substitution
  ) -> bool
  {
    let multiplicity = top_variable.multiplicity;
    if multiplicity == 1 {
      if self.current.size == 1 && self.current.max_multiplicity() == 1 {
        let d = self.current.get_sole_dag_node();
        if d.leq_sort(&top_variable.sort) {
          solution.bind(top_variable.index, d);
          self.current.clear();
          return true;
        }
      } else {
        {
          let t = ACUDagNode::new(&self.top_symbol, self.current);
          let index = self.current.compute_base_sort(self.top_symbol.as_ref());
          if index_leq_sort(index, self.top_variable.sort.as_ref()) {
            if subject.is_reduced && self.top_symbol.sort_constraint_free() {
              t.sort_index = index;
              t.is_reduced = true;
            }
            solution.bind(self.top_variable.index, t);
            self.current.clear();
            return true;
          }
        }
        if match_at_top && matched_multiplicity >= 1 {
          let mut j = ACU_SlowIter::new(self.current);
          loop {
            let d = j.get_dag_node();
            if d.leq(self.top_variable.sort) {
              solution.bind(self.top_variable.index, d);
              self.current.delete_mult(j, 1);
              matched_multiplicity += 1;
              return true;
            }
            if !j.next() {
              break;
            }
          }
        }
      }
    } else {
      if match_at_top {
        let d = make_high_multiplicity_assignment(multiplicity, top_variable.sort, self.current);
        if d != 0 {
          solution.bind(top_variable.index, d);
          matched_multiplicity = 2;
          return true;
        }
      } else {
        let size = self.current.size;
        if size == 1 && self.current.get_sole_multiplicity() == multiplicity {
          let d = self.current.get_sole_dag_node();
          if d.leq(top_variable.sort) {
            solution.bind(top_variable.index, d);
            self.current.clear();
            return true;
          }
          return false;
        }
      
        let d = ACU_DagNode::new(top_symbol, self.current.size, ACU_DagNode::ASSIGNMENT);
        let mut dest = d.arg_array.iter_mut();
        let mut i = ACU_SlowIter::new(self.current);
        loop {
          let m = i.get_multiplicity();
          if m % multiplicity != 0 {
            return false;
          }
          dest.dag_node = i.get_dag_node();
          dest.multiplicity = m / multiplicity;
          dest.next();
          if !i.next() {
            break;
          }
        }
      }
    }
    false
  }
  // unique_collapse_match
  // multiway_collapse_match

}

impl LhsAutomaton for ACULHSAutomaton {

  /// Returns the outcome with an optional subproblem.
  fn match_(
    &mut self,
    subject       : RcDagNode,
    solution      : &mut Substitution,
    mut extension_info: Option<&mut dyn ExtensionInfo>,
  ) -> (bool, MaybeSubproblem)
  {
    let mut returned_subproblem: MaybeSubproblem = None;


    if subject.get_ref().symbol() != self.top_symbol() {
      if self.collapse_possible {
        return self.collapse_match(subject.clone(), solution, extension_info)
      }
      return (false, None);
    }

    assert_eq!(self.match_at_top, extension_info.is_some(), "match_at_top disagreement");

    // Todo: What is the point of this?
    // returned_subproblem  = 0;

    if let Some(s) = subject.get_ref().as_any().downcast_mut::<ACUDagNode>() {
      //	Check to see if we should use red-black matcher.
      if let ACUArguments::Tree(t) = &s.args {
        if self.tree_match_ok {
          let (r, subproblem) = self.tree_match(t, solution, &mut extension_info);
          // r == true || r == false
          match r {
            Outcome::Failure => { return (false, None); }
            Outcome::Success => { return (true, subproblem);  }
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
        return (false, None);
      }

      if extension_info.is_none() {
        //	Matching without extension special cases:
        if self.last_unbound_variable == -1 /* NONE */ {
          // Todo: Implement `compute_total_multiplicity()`
          if self.total_non_ground_aliens_multiplicity != self.compute_total_multiplicity() {
            return (false, None);
          }
          if self.total_non_ground_aliens_multiplicity == 0 {
            return (true, None);
          }
          if self.match_strategy == MatchStrategy::AliensOnly {
            // Todo: Implement `aliens_only_match()`
            return self.aliens_only_match(s, solution, returned_subproblem);
          }
        }
      }


    } else {
      panic!("ACULHSAutomaton::match  called with non ACU DagNode. This is a bug.");
    }

    (false, None)
  }

}
