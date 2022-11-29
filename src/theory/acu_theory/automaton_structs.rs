/*!

A collection of structs used in LhsAutomaton/RhsAutomaton.

*/


use crate::Sort;

use crate::theory::{
  AssociativeSymbolStructure,
  LhsAutomaton,
  Term
};

#[derive(PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum MatchStrategy {
  //	Ground out case: no extension and everything grounds out;
  //	return fail if any subjects left.
  GroundOut,

  //	Lone variable case: no extension, no aliens, 1 unbound variable which is forced
  //	to match remaining subjects.
  LoneVariable,

  //	Aliens only case: no extension, all top variables are guaranteed to be
  //	bound on entry to matcher. Alien sequence will be ordered:
  //		(1) to optimize strong constraint propagation (match independent aliens)
  //		(2) to "force" as many aliens as possible (subsumptive aliens)
  //		(3) to optimize weak constraint propagation (aliens left over)
  //	The field nrIndependents holds the total number in parts (1) and (2) of
  //	alien sequence.
  AliensOnly,

  //	Greedy case: the intersection of each subterm's variables with union of
  //	its context variables and the condition variables contains only variables
  //	guaranteed to be bound on entry to matcher. Alien sequence will be ordered
  //	to maximize the chance that greedy failure is true failure. The field
  //	nrIndependents holds the number of aliens for which greedy failure is true
  //	failure. The top variable sequence will be ordered to maximize the chance of
  //	finding a greedy match, and in the pure greedy case, to maximize the chance
  //	that pure greedy failure is true failure.
  Greedy,

  //	Full case: if other cases don't apply. Alien sequence will be ordered to
  //	optimize solve-time weak constraint propagation. The field nrIndependents
  //	is unused.
  Full
}


pub(crate) struct TopVariable<'s> {
  pub(crate) index        : u32,
  pub(crate) multiplicity : u32,
  sort         : &'s Sort<'s>,
  upper_bound  : u32,
  structure    : AssociativeSymbolStructure,
  take_identity: bool,
  abstracted   : Box<dyn LhsAutomaton>,          // automaton for abstracted term

  //	Data storage for match-time use
  pub(crate) previous_unbound: u32,
  first_subject   : u32,
  subject_count   : u32,
}


pub(crate) struct GroundAlien<'t> {
  pub(crate) term: &'t dyn Term,
  pub(crate) multiplicity: u32
}


pub(crate) struct NonGroundAlien<'t> {
  pub(crate) term: Option<&'t dyn Term>,
  pub(crate) multiplicity: u32,
  pub(crate) lhs_automaton: Box<dyn LhsAutomaton>
}


pub(crate) struct Subject {
  pub(crate) multiplicity: u32,
  pub(crate) assignee: u32
}
