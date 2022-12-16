/*!

Trait for DAG nodes.

 */

use std::any::Any;
use std::cell::Cell;
use std::cmp::Ordering;
use std::rc::Rc;

use dyn_clone::{clone_trait_object, DynClone};
use reffers::rc1::Strong;

use crate::{
  Sort,
  theory::Symbol,
  OrderingValue
};
use crate::sort::{RcSort, SpecialSorts};
use crate::theory::Outcome;

// pub type BcDagNode = Box<Cell<DagNode>>;
pub type BcDagNode = Box<dyn DagNode>;
// Todo: Replace `&DagNode` with `RcDagNode`
pub type RcDagNode = Strong<dyn DagNode>;

/// This struct owns the DagNode. If we just want a reference, we use a tuple `(dag_node.as_ref(), multiplicity)`.
#[derive(Clone)]
pub struct DagPair {
  pub(crate) dag_node    : RcDagNode,
  pub(crate) multiplicity: u32
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum DagNodeFlag{
  Reduced      = 1,  // Reduced up to strategy by equations
  Copied       = 2,  // Copied in current copy operation; copyPointer valid
  Unrewritable = 4,  // Reduced and not rewritable by rules
  Unstackable  = 8,  // Unrewritable and all subterms unstackable or frozen
  Ground   = 16, // No variables occur below this node
  HashValid    = 32, // Node has a valid hash value (storage is theory dependent)
}
impl DagNodeFlag {
  // We can share the same bit as UNREWRITABLE for this flag since the rule rewriting strategy that needs UNREWRITABLE
  // never be combined with variant narrowing. Implemented as associated type since Rust does not allow variant aliases.
  //    IRREDUCIBLE_BY_VARIANT_EQUATIONS = 4
  pub const IrreducibleByVariantEquations: DagNodeFlag = DagNodeFlag::Unrewritable;
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DagNodeFlags(u32);
impl DagNodeFlags{
  fn  is_reduced(&self) -> bool {
    (self & DagNodeFlag::Reduced) != 0
  }
  fn  is_copied(&self) -> bool {
    (self & DagNodeFlag::Copied) != 0
  }
  fn  is_unrewritable(&self) -> bool {
    (self & DagNodeFlag::Unrewritable) != 0
  }
  fn  is_unstackable(&self) -> bool {
    (self & DagNodeFlag::Unstackable) != 0
  }
  fn  is_ground(&self) -> bool {
    (self & DagNodeFlag::Ground) != 0
  }
  fn  is_hash_valid(&self) -> bool {
    (self & DagNodeFlag::HashValid) != 0
  }
}

// Todo: Maude puts `copyPointer` and `top_symbol` in a union for optimization.
pub trait DagNode: DynClone {
  /// Gives the top symbol of this term.
  fn symbol(&self)         -> &Symbol;
  fn symbol_mut(&mut self) -> &mut Symbol;

  /// Returns an iterator over `(DagNode, u32)` pairs for the arguments.
  fn iter_args(&self) -> Box<dyn Iterator<Item=(RcDagNode, u32)>> ;

  /// Defines a partial order on `DagNode`s. Unlike the `Ord`/`PartialOrd` implementation, this method also compares
  /// the arguments.
  fn compare(&self, other: &Self) -> Ordering {
    let symbol_order = self.cmp(other);

    match symbol_order {
      Ordering::Equal => self.compare_arguments(other),
      _               => symbol_order
    }
  }

  fn compare_arguments(&self, &other: Self) -> Ordering;


  fn get_sort(&self) -> RcSort;

  fn leq_sort(&self, sort: &Sort) -> bool {
    self.get_sort().get_ref().leq(sort)
  }

  fn set_sort_index(&mut self, sort_index: u32);

  fn check_sort(&mut self, bound_sort: RcSort) -> (Outcome, MaybeSubproblem) {
    if self.get_sort() != SpecialSorts::SortUnknown {
      return (self.leq_sort(bound_sort.as_ref()).into(), None);
    }

    self.symbol_mut().compute_base_sort(self);
    if self.leq_sort(bound_sort.as_ref()) {
      if !self.symbol().sort_constraint_free() {
        self.set_sort_index( SpecialSorts::SortUnknown.into() );
      }
    } else {
      if self.symbol().sort_constraint_free() {
        return (Outcome::Failure, None);
      }
      self.set_sort_index( SpecialSorts::SortUnknown.into() );
      // Todo: Implement `SortCheckSubproblem`.
      // let returned_subproblem = SortCheckSubproblem::new(this, bound_sort);
      // return (Outcome::Success, Some(returned_subproblem))
    }

    return (Outcome::Success, None);
  }

  /// The number of arguments.
  fn len(&self) -> u32;

  fn as_any(&self) -> &dyn Any;

  fn flags(&self) -> DagNodeFlags;

  // region Flag Manipulation
  fn  is_reduced(&self) -> bool {
    (self.flags() as u32 & DagNodeFlag::Reduced) != 0
  }
  fn  is_copied(&self) -> bool {
    (self.flags() as u32 & DagNodeFlag::Copied) != 0
  }
  fn  is_unrewritable(&self) -> bool {
    (self.flags() as u32 & DagNodeFlag::Unrewritable) != 0
  }
  fn  is_unstackable(&self) -> bool {
    (self.flags() as u32 & DagNodeFlag::Unstackable) != 0
  }
  fn  is_ground(&self) -> bool {
    (self.flags() as u32 & DagNodeFlag::Ground) != 0
  }
  fn  is_hash_valid(&self) -> bool {
    (self.flags() as u32 & DagNodeFlag::HashValid) != 0
  }
  // endregion

}

clone_trait_object!(DagNode);

impl Eq for dyn DagNode {}

impl PartialEq<Self> for dyn DagNode {
  fn eq(&self, other: &Self) -> bool {
    self.symbol().eq(other.symbol())
  }
}


impl PartialOrd<Self> for dyn DagNode {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for dyn DagNode {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.symbol().cmp(&other.symbol())
  }
}
