/*!

The `Term` trait is implemented by things that can be nodes in the expression tree. That is, an expression tree is a
term, and each subexpression is a term. The algorithms do not operate on expression trees (terms). Instead, the
algorithms operate on a directed acyclic graph (DAG) is constructed from the tree. Thus, for each `Term` type, there
is a corresponding `DagNode` type. However, because of structural sharing, the node instances themselves are not in
1-to-1 correspondence.

Types Implementing `Term`:
    `ACUTerm`
    `FreeTerm`
    `VariableTerm`

*/

use std::{
  any::Any,
  cmp::Ordering,
  collections::{
    hash_map::Entry,
    HashMap
  },
  fmt::{Display, Formatter},
  hash::{Hash, Hasher},
  net::Shutdown::Write,
  ptr::addr_of,
  rc::Rc,
};
use std::hash::BuildHasher;

use crate::{abstractions::{
  FastHasher,
  FastHasherBuilder,
  NatSet,
  RcCell,
  Set,
}, core::{
  format::{
    FormatStyle,
    Formattable
  },
  OrderingValue,
  sort::{
    RcConnectedComponent,
    SpecialSort
  },
  substitution::Substitution,
  TermBag,
  VariableInfo,
}, NONE, theory::variable::VariableTerm, UNDEFINED};
use crate::core::automata::RHSBuilder;

use super::{
  dag_node_flags,
  DagNode,
  DagNodeFlag,
  LHSAutomaton,
  NodeList,
  RcDagNode,
  RcLHSAutomaton,
  RcSymbol,
  Symbol,
  SymbolSet,
};

pub type MaybeTerm = Option<RcTerm>;
pub type RcTerm    = RcCell<dyn Term>;
pub type TermSet   = Set<dyn Term>;
// ToDo: Replace with analog of `TermHashSet`.
pub type NodeCache = HashMap<u32, RcDagNode, FastHasherBuilder>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TermKind {
  Free,
  Bound,
  Ground,
  NonGround
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(crate) enum TermAttribute {
  ///	A subterm is stable if its top symbol cannot change under instantiation.
  Stable = 1,

  ///	A subterm is in an eager context if the path to its root contains only
  ///	eagerly evaluated positions.
  EagerContext = 2,

  ///	A subterm "honors ground out match" if its matching algorithm guarantees
  ///	never to to return a matching subproblem when all the terms variables
  ///	are already bound.
  HonorsGroundOutMatch = 4
}

/*
One way to deal with a lack of trait data members is to have a struct containing the shared members and then
either
  1. have a macro that implements the getters and setters, or
  2. have a trait-level getter for the struct that is implemented in every implementor, and have
     shared-implementation at the trait level by using the getter in the `impl Trait`.
We choose the second option.
*/
pub struct TermMembers {
  pub(crate) top_symbol         : RcSymbol,
  /// The handles (indices) for the variable terms that occur in this term or its descendants
  pub(crate) occurs_set         : NatSet,
  pub(crate) context_set        : NatSet,
  pub(crate) collapse_set       : SymbolSet,
  pub(crate) attributes         : u8,
  pub(crate) sort_index         : i32, //i16,
  pub(crate) connected_component: RcConnectedComponent,
  pub(crate) save_index         : i32, // NoneIndex = -1
  // pub(crate) hash_value         : u32,
  pub(crate) cached_size        : i32,

  // Static Members

  // pub(crate) static sub_dags : NodeList,
  // pub(crate) static converted: TermSet,
  // This is the HashMap of dag nodes that allows structural sharing. Maude implements it with two structures. It is
  // reset on each call to term2dag and is only used during dagification. It should be able to be replaced with a
  // parameter to `dagify()` in all cases.
  // Note: `dagify2()` is the theory specific part of `dagify()`.

  // pub(crate) static set_sort_info_flag: bool
  // This is only used twice:
  //   1. CachedDag::getDag()
  //   2. SubtermTask::SubtermTask
  // It should be able to be replaced with a parameter to `dagify()` in all cases.
}

impl TermMembers {
  pub fn new(symbol: RcSymbol) -> TermMembers {
    TermMembers {
      top_symbol         : symbol,
      occurs_set         : Default::default(),
      context_set        : Default::default(),
      collapse_set       : Default::default(),
      attributes         : 0,
      sort_index         : SpecialSort::Unknown as i32,
      connected_component: Default::default(),
      save_index         : 0,
      // hash_value         : 0,
      cached_size        : UNDEFINED,
    }
  }
}


pub trait Term: Formattable {

  fn as_any(&self)         -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn as_ptr(&self)         -> *const dyn Term;
  fn semantic_hash(&self) -> u32;
  /// Normalizes the term, returning the computed hash and `true` if the normalization changed
  /// the term or `false` otherwise.
  fn normalize(&mut self, full: bool) -> (u32, bool);


  // region Accessors

  /// Gives the top symbol of this term.
  #[inline(always)]
  fn symbol(&self) -> RcSymbol {
    self.term_members().top_symbol.clone()
  }

  /// Access to data members. This allows shared implementation in the trait implementation rather than generic
  /// implementation being reproduced for every implementor of the trait.
  fn term_members(&self) -> &TermMembers;
  fn term_members_mut(&mut self) -> &mut TermMembers;

  /// Is the term stable?
  #[inline(always)]
  fn is_stable(&self) -> bool {
    self.term_members().attributes & TermAttribute::Stable as u8 != 0
  }

  /// A subterm "honors ground out match" if its matching algorithm guarantees never to return a matching subproblem
  /// when all the terms variables are already bound.
  #[inline(always)]
  fn honors_ground_out_match(&self) -> bool {
    self.term_members().attributes & TermAttribute::HonorsGroundOutMatch as u8 != 0
  }

  #[inline(always)]
  fn set_honors_ground_out_match(&mut self, value: bool) {
    let mut members = self.term_members_mut();
    if value {
      members.attributes = members.attributes | TermAttribute::HonorsGroundOutMatch as u8;
    } else {
      members.attributes = members.attributes & !(TermAttribute::HonorsGroundOutMatch as u8);
    }
  }

  #[inline(always)]
  fn is_eager_context(&self) -> bool {
    self.term_members().attributes & TermAttribute::EagerContext as u8 != 0
  }

  #[inline(always)]
  fn is_variable(&self) -> bool {
    self.symbol().is_variable()
  }

  #[inline(always)]
  fn ground(&self) -> bool {
    self.term_members().occurs_set.is_empty()
  }

  /// The handles (indices) for the variable terms that occur in this term or its descendants
  #[inline(always)]
  fn occurs_below(&self) -> &NatSet {
    &self.term_members().occurs_set
  }

  #[inline(always)]
  fn occurs_below_mut(&mut self) -> &mut NatSet {
    &mut self.term_members_mut().occurs_set
  }

  #[inline(always)]
  fn occurs_in_context(&self) -> &NatSet {
    &self.term_members().context_set
  }

  #[inline(always)]
  fn occurs_in_context_mut(&mut self) -> &mut NatSet {
    &mut self.term_members_mut().context_set
  }

  #[inline(always)]
  fn collapse_symbols(&self) -> &SymbolSet {
    &self.term_members().collapse_set
  }

  /// Returns an iterator over the arguments of the term.
  fn iter_args(&self) -> Box<dyn Iterator<Item=RcTerm> + '_>;


  /// Compute the number of nodes in the term tree.
  fn compute_size(&mut self) -> i32 {
    if self.term_members().cached_size != UNDEFINED {
      self.term_members().cached_size
    }
    else {
      let mut size = 1; // Count self.
      for arg in self.iter_args() {
        size += arg.borrow_mut().compute_size();
      }
      self.term_members_mut().cached_size = size;
      size
    }
  }

  fn set_sort_info(&mut self, connected_component: RcConnectedComponent, sort_index: i32) {
    let mut members = self.term_members_mut();
    members.connected_component = connected_component;
    members.sort_index = sort_index;
  }

  /// Sets the given attribute (to true)
  fn set_attribute(&mut self, attribute: TermAttribute) {
    self.term_members_mut().attributes |= attribute as u8;
  }

  /// Resets the given attribute (to false)
  fn reset_attribute(&mut self, attribute: TermAttribute) {
    self.term_members_mut().attributes &= !(attribute as u8);
  }

  #[inline(always)]
  fn sort_index(&self) -> i32 {
    self.term_members().sort_index
  }

  #[inline(always)]
  fn connected_component(&self) -> RcConnectedComponent {
    self.term_members().connected_component.clone()
  }



  // endregion

  // region Comparison Functions

  /// Downcasts to concrete implementing type
  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering;

  #[inline(always)]
  fn compare_dag_node(&self, other: &dyn DagNode) -> Ordering {
    if self.symbol().semantic_hash() == other.symbol().semantic_hash() {
      self.compare_dag_arguments(other)
    } else {
      self.symbol().compare(other.symbol().as_ref())
    }
  }

  /// Downcasts to Self
  fn compare_dag_arguments(&self, other: &dyn DagNode) -> Ordering;


  fn partial_compare(&self, partial_substitution: &mut Substitution, other: &dyn DagNode) -> OrderingValue {
    if !self.is_stable() {
      // Only used for `VariableTerm`
      return self.partial_compare_unstable(partial_substitution, other);
    }

    if Rc::ptr_eq(&self.symbol(), &other.symbol()) {
      // Only used for `FreeTerm`
      return self.partial_compare_arguments(partial_substitution, other);
    }

    if self.symbol().compare(other.symbol().as_ref())  == Ordering::Less {
      OrderingValue::Less
    } else {
      OrderingValue::Greater
    }
  }

  #[inline(always)]
  fn compare(&self, other: &dyn Term) -> Ordering {
    let other_symbol = other.symbol();
    let r = self.symbol().compare(other_symbol.as_ref());
    if r == Ordering::Equal {
      return self.compare_term_arguments(other);
    }
    return r;
  }

  /// Overridden in `VariableTerm`
  fn partial_compare_unstable(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> OrderingValue {
    OrderingValue::Unknown
  }

  /// Overridden in `FreeTerm`
  fn partial_compare_arguments(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> OrderingValue {
    OrderingValue::Unknown
  }

  // endregion

  // region DAG Creation
  /// Create a directed acyclic graph from this term. This is a convenience method to be an entry point for `dagify(…)`.
  #[inline(always)]
  fn make_dag(&self) -> RcDagNode {
    let mut node_cache = NodeCache::with_hasher(FastHasherBuilder::new());
    self.dagify(&mut node_cache, false)
  }

  /// Create a directed acyclic graph from this term. This trait-level implemented function takes care of structural
  /// sharing. Each implementing type will supply its own implementation of `dagify_aux(…)`, which recursively
  /// calls `dagify(…)` on its children and then converts itself to a type implementing DagNode, returning `RcDagNode`.
  fn dagify(&self, sub_dags: &mut NodeCache, set_sort_info: bool) -> RcDagNode {
    // let self_ptr = self.as_ptr().addr();
    let self_hash = self.semantic_hash();

    if let Entry::Occupied(occupied_entry) = sub_dags.entry(self.semantic_hash()) {
      let entry = occupied_entry.get();
      return entry.clone();
    }

    let d = self.dagify_aux(sub_dags, set_sort_info);
    if set_sort_info {
      assert_ne!(self.term_members().sort_index, SpecialSort::Unknown as i32, "Missing sort info");
      let mut d = d.borrow_mut();
      d.set_sort_index(self.term_members().sort_index);
      d.set_flags(DagNodeFlag::Reduced.into());
    }
    sub_dags.insert(self_hash, d.clone());

    d
  }

  /// Create a directed acyclic graph from this term. This method has the implementation-specific stuff.
  fn dagify_aux(&self, sub_dags: &mut NodeCache, set_sort_info: bool) -> RcDagNode;

  // endregion

  // region Compiler-related Functions //

  /// Compiles the LHS automaton, returning the tuple `(lhs_automaton, subproblem_likely): (RcLHSAutomaton, bool)`
  fn compile_lhs(
    &self,
    match_at_top  : bool,
    variable_info : &VariableInfo,
    bound_uniquely: &mut NatSet,
  ) -> (RcLHSAutomaton, bool);

  /// The theory-dependent part of `compile_rhs` called by `term_compiler::compile_rhs(…)`. Returns
  /// the `save_index`.
  fn compile_rhs_aux(
    &mut self,
    builder        : &mut RHSBuilder,
    variable_info  : &VariableInfo,
    available_terms: &mut TermBag,
    eager_context  : bool
  ) -> i32;

  // A subterm "honors ground out match" if its matching algorithm guarantees never to return a matching subproblem
  // when all the terms variables are already bound.
  fn will_ground_out_match(&self, bound_uniquely: &NatSet) -> bool {
    self.honors_ground_out_match() && bound_uniquely.is_superset(&self.term_members().occurs_set)
  }

  fn analyse_constraint_propagation(&mut self, bound_uniquely: &mut NatSet);

  fn analyse_collapses(&mut self) {
    for arg in &mut self.iter_args() {
      arg.borrow_mut().analyse_collapses();
    }

    if !self.is_variable() && self.collapse_symbols().is_empty() {
      self.set_attribute(TermAttribute::Stable);
    }
  }

  /// This is the theory-specific part of `find_available_terms`
  fn find_available_terms_aux(&self, available_terms: &mut TermBag, eager_context: bool, at_top: bool);

  /// Computes and updates the set of variables that occur in the context of a term and its
  /// subterms. The "context" of a term refers to the rest of the term in which it occurs (its
  /// parent term and sibling subterms).
  fn determine_context_variables(&mut self) {
    // Used to defer mutation of self while immutable borrow of self is held by `iter_args`.
    let mut context_set = NatSet::default();

    for t in self.iter_args() {
      // Insert parent's context set
      context_set.union_in_place(t.borrow().occurs_in_context());
      // self.occurs_in_context_mut().union_in_place(t.borrow().occurs_in_context());

      for u in self.iter_args() {
        if *u.borrow() != *t.borrow() {
          // Insert sibling's occurs set
          context_set.union_in_place(u.borrow().occurs_below());
          // self.occurs_in_context_mut().union_in_place(u.borrow().occurs_below());
        }
      }

      t.borrow_mut().determine_context_variables();
    }
    self.occurs_in_context_mut().union_in_place(&context_set);
  }

  fn insert_abstraction_variables(&mut self, variable_info: &mut VariableInfo) {
    self.set_honors_ground_out_match(true);
    let mut hgom = true;

    for t in self.iter_args() {
      hgom &= {
        let mut term = t.borrow_mut();
        term.insert_abstraction_variables(variable_info);
        term.honors_ground_out_match()
      };
    }
    if !hgom {
      self.set_honors_ground_out_match(false);
    }
  }

  /// This method populates the sort information for the term and its subterms based on their
  /// symbol's sort declarations, validating them against the symbol's expected input and output
  /// types (domain and range components). (This is a method on `Symbol` in Maude.)
  fn fill_in_sort_info(&mut self) {
    let symbol    = self.symbol();
    let component = symbol.sort_table().range_component(); // should be const
    // assert!(component.is_some(), "couldn't get component");

    if symbol.arity() == 0 {
      self.set_sort_info(
        component.clone(),
        symbol.sort_table().traverse(0, 0)
      ); // HACK
      return;
    }

    let mut step = 0;
    // let mut seen_args_count = 0;

    for t in &mut self.iter_args() {
      let mut term = t.borrow_mut();
      term.fill_in_sort_info();
      // ToDo: Restore this assert.
      // debug_assert_eq!(
      //   term.get_component(),
      //   symbol.domain_component(seen_args_count),
      //   "component error on arg {} while computing sort of {}",
      //   seen_args_count,
      //   self
      // );
      step = symbol.sort_table().traverse(step as usize, term.sort_index() as usize);
      // seen_args_count += 1;
    }

    // ToDo: Restore this assert.
    // debug_assert_eq!(seen_args_count, seen_args_count, "bad # of args for op");
    self.set_sort_info(component, step);
  }

  // endregion
}

// region trait impls for Term
impl Display for dyn Term{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.repr(FormatStyle::Default))
  }
}


// ToDo: Revisit whether `semantic_hash` is appropriate for the `Hash` trait.
// Use the `Term::compute_hash(…)` hash for `HashSet`s and friends.
impl Hash for dyn Term {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u32(self.compute_hash())
  }
}

impl PartialEq for dyn Term {
  fn eq(&self, other: &Self) -> bool {
    self.semantic_hash() == other.semantic_hash()
  }
}

impl Eq for dyn Term{}
// endregion

/// Recursively collects the indices and occurs sets of this term and its descendants.
///
/// This is a free function, because we want it wrapped in the Rc so that when we call `variable_to_index()`
/// it's possible to add the Rc to the indices vector.
pub fn index_variables(term: RcTerm, indices: &mut VariableInfo) {
  // This condition needs to check an RcTerm for a VariableTerm
  if term.borrow().is_variable() {
    // This call needs an RcTerm
    let index = indices.variable_to_index(term.clone());
    {
      let mut term = term.borrow_mut();
      let variable_term = term.as_any_mut().downcast_mut::<VariableTerm>().unwrap();

      // This call needs a mutable VariableTerm
      variable_term.index = index;
      variable_term.occurs_below_mut().insert(index as usize);
    }
  } else {
    let mut term_mut = term.borrow_mut();
    // Accumulate in a local variable, because the iterator holds a mutable borrow.
    let mut occurs_below = NatSet::new();
    for arg in term_mut.iter_args() {
      index_variables(arg.clone(), indices);
      // Accumulate the set of variables that occur under this symbol.
      occurs_below.union_in_place(
                &arg.borrow()
                    .occurs_below()
              );
    }
    term_mut.occurs_below_mut().union_in_place(&occurs_below);
  }
}


/// Recursively collects the terms in a set for structural sharing.
///
/// This is a free function, because we want it wrapped in the Rc so that when we call `find_available_terms()`
/// it's possible to add the Rc to the term set.
pub fn find_available_terms(term: RcTerm, available_terms: &mut TermBag, eager_context: bool, at_top: bool) {
  if term.borrow().ground() {
    return;
  }

  if !at_top {
    available_terms.insert_matched_term(term.clone(), eager_context);
  }

  // Now do theory-specific stuff
  term.borrow().find_available_terms_aux(available_terms, eager_context, at_top);
}
