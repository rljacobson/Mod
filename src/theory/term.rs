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
  cmp::Ordering,
  rc::Rc,
  any::Any
};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::net::Shutdown::Write;
use std::ptr::addr_of;

use crate::{
  core::{
    Substitution,
    OrderingValue,
    RcConnectedComponent,
    NatSet
  },
  abstractions::{RcCell, Set, FastHasher, FastHasherBuilder},
  theory::{
    RcSymbol,
    DagNode,
    NodeList,
    Symbol,
    symbol::SymbolSet
  }
};
use crate::core::SpecialSort;
use crate::theory::{DagNodeFlag, dag_node_flags, RcDagNode};


pub type RcTerm = RcCell<dyn Term>;
pub type TermSet = Set<dyn Term>;
pub type NodeCache<'s> = HashMap<u32, RcDagNode, FastHasherBuilder>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TermKind {
  Free,
  Bound,
  Ground,
  NonGround
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(crate) enum TermFlags {
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
  pub(crate) occurs_set         : NatSet,
  pub(crate) context_set        : NatSet,
  pub(crate) collapse_set       : SymbolSet,
  pub(crate) flags              : u8,
  pub(crate) sort_index         : i32, //i16,
  pub(crate) connected_component: RcConnectedComponent,
  pub(crate) save_index         : i32,
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
      flags              : 0,
      sort_index         : SpecialSort::Unknown as i32,
      connected_component: Default::default(),
      save_index         : 0,
      // hash_value         : 0,
      cached_size        : 0,
    }
  }
}


pub trait Term {
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
    self.term_members().flags & TermFlags::Stable as u8 != 0
  }

  /// Downcasts to concrete implementing type
  fn compare_term_arguments(&self, other: &dyn Term) -> Ordering;

  #[inline(always)]
  fn compare_dag_node(&self, other: &dyn DagNode) -> Ordering {
    if self.symbol().get_hash_value() == other.symbol().get_hash_value() {
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

  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;

  /// Overridden in `VariableTerm`
  fn partial_compare_unstable(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> OrderingValue {
    OrderingValue::Unknown
  }

  /// Overridden in `FreeTerm`
  fn partial_compare_arguments(&self, _partial_substitution: &mut Substitution, _other: &dyn DagNode) -> OrderingValue {
    OrderingValue::Unknown
  }

  /// Create a directed acyclic graph from this term. This is a convenience method to be an entry point for `dagify(…)`.
  #[inline(always)]
  fn make_dag(&self) -> RcDagNode {
    let mut node_cache = NodeCache::with_hasher(FastHasherBuilder);
    self.dagify(&mut node_cache, false)
  }

  /// Create a directed acyclic graph from this term. This trait-level implemented function takes care of structural
  /// sharing. Each implementing type will supply its own implementation of `dagify_aux(…)`, which recursively
  /// calls `dagify(…)` on its children and then converts itself to a type implementing DagNode, returning `RcDagNode`.
  fn dagify(&self, sub_dags: &mut NodeCache, set_sort_info: bool) -> RcDagNode {
    // let self_ptr = self.as_ptr().addr();
    let self_hash = self.compute_hash();

    if let Entry::Occupied(occupied_entry) = sub_dags.entry(self.compute_hash()) {
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


  fn as_ptr(&self) -> *const dyn Term;

  fn repr(&self) -> String;

  fn compute_hash(&self) -> u32;

  /// Normalizes the term, returning the computed hash and `true` if the normalization changed
  /// the term or `false` otherwise.
  fn normalize(&mut self, full: bool) -> (u32, bool);

}

impl Display for dyn Term{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.repr())
  }
}


/// Use the `Term::compute_hash(…)` hash for `HashSet`s and friends.
impl Hash for dyn Term {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u32(self.compute_hash())
  }
}

impl PartialEq for dyn Term {
  fn eq(&self, other: &Self) -> bool {
    self.compute_hash() == other.compute_hash()
  }
}

impl Eq for dyn Term{}
