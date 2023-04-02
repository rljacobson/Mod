/*!

Trait for DAG nodes.

 */

use std::any::Any;
use std::cmp::Ordering;
use std::rc::Rc;

use reffers::rc1::{Strong, Weak};
use dyn_clone::{clone_trait_object, DynClone};

use crate::{
  core::{
    Sort,
    RcSort, 
    SpecialSort
  },
  theory::{
    Symbol,
    Outcome,
    MaybeSubproblem
  }
};


// pub type BcDagNode = Box<Cell<DagNode>>;
pub type BcDagNode = Box<dyn DagNode>;
// Todo: `Rc<dyn DagNode>` with `Strong<dyn DagNode>`. Using `Strong` prevents `DagNod` from being a trait object for
// some reason.
pub type RcDagNode = Rc<dyn DagNode>;

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
  Ground       = 16, // No variables occur below this node
  HashValid    = 32, // Node has a valid hash value (storage is theory dependent)
}

impl DagNodeFlag {
  // We can share the same bit as UNREWRITABLE for this flag since the rule rewriting strategy that needs UNREWRITABLE
  // never be combined with variant narrowing. Implemented as associated type since Rust does not allow variant aliases.
  //    IRREDUCIBLE_BY_VARIANT_EQUATIONS = 4
  pub const IRREDUCIBLE_BY_VARIANT_EQUATIONS: DagNodeFlag = DagNodeFlag::Unrewritable;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DagNodeFlags(u32);

impl From<DagNodeFlag> for DagNodeFlags {
    fn from(value: DagNodeFlag) -> Self {
        Self(value as u32)
    }
}

impl DagNodeFlags{
  fn  is_reduced(&self) -> bool {
    (self.0 & DagNodeFlag::Reduced as u32) != 0
  }
  fn  is_copied(&self) -> bool {
    (self.0 & DagNodeFlag::Copied as u32) != 0
  }
  fn  is_unrewritable(&self) -> bool {
    (self.0 & DagNodeFlag::Unrewritable as u32) != 0
  }
  fn  is_unstackable(&self) -> bool {
    (self.0 & DagNodeFlag::Unstackable as u32) != 0
  }
  fn  is_ground(&self) -> bool {
    (self.0 & DagNodeFlag::Ground as u32) != 0
  }
  fn  is_hash_valid(&self) -> bool {
    (self.0 & DagNodeFlag::HashValid as u32) != 0
  }
}

// Todo: Maude puts `copyPointer` and `top_symbol` in a union for optimization.
pub trait DagNode {
  /// Gives the top symbol of this term.
  fn symbol(&self)         -> &dyn Symbol;
  fn symbol_mut(&mut self) -> &mut dyn Symbol;

  /// Returns an iterator over `(RcDagNode, u32)` pairs for the arguments.
  fn iter_args(&self) -> Box<dyn Iterator<Item=(RcDagNode, u32)>> ;

  /// Defines a partial order on `DagNode`s. Unlike the `Ord`/`PartialOrd` implementation, this method also compares
  /// the arguments.
  fn compare(&self, other: &dyn DagNode) -> Ordering {
    // let symbol_order = self.cmp(other);
    let s = self.symbol();
    let symbol_order = //Ord::cmp(s, other.symbol());
      s.get_hash_value().cmp(&other.symbol().get_hash_value());

    match symbol_order {
      Ordering::Equal => self.compare_arguments(other),
      _               => symbol_order
    }
  }

  fn compare_arguments(&self, other: &dyn DagNode) -> Ordering;


  fn get_sort(&self) -> RcSort;

  fn leq_sort(&self, sort: &Sort) -> bool {
    self.get_sort().get_ref().leq(sort)
  }

  fn set_sort_index(&mut self, sort_index: i32);
  fn get_sort_index(&self) -> i32;

  fn compute_base_sort(&self) -> i32;

  fn check_sort(&mut self, bound_sort: RcSort) -> (Outcome, MaybeSubproblem) where Self: Sized {
    if *self.get_sort().get_ref().as_ref() != SpecialSort::Unknown {
      return (self.leq_sort(bound_sort.get_ref().as_ref()).into(), None);
    }

    // This is a weird code smell.
    // self.symbol_mut().compute_base_sort(self);
    // The ACUSymbol just turns around and calls `compute_base_sort` on the owning `DagNode`.
    let sort_index = self.compute_base_sort();
    self.set_sort_index(sort_index);


    if self.leq_sort(bound_sort.get_ref().as_ref()) {
      if !self.symbol().sort_constraint_free() {
        self.set_sort_index( SpecialSort::Unknown as i32 );
      }
    } else {
      if self.symbol().sort_constraint_free() {
        return (Outcome::Failure, None);
      }
      self.set_sort_index( SpecialSort::Unknown as i32 );
      // Todo: Implement `SortCheckSubproblem`.
      // let returned_subproblem = SortCheckSubproblem::new(this, bound_sort);
      // return (Outcome::Success, Some(returned_subproblem))
    }

    return (Outcome::Success, None);
  }

  /// The number of arguments.
  fn len(&self) -> usize;

  fn as_any(&self) -> &dyn Any;

  fn flags(&self) -> DagNodeFlags;

  // region Flag Manipulation
  fn  is_reduced(&self) -> bool {
    (self.flags().0 & DagNodeFlag::Reduced as u32) != 0
  }
  fn  is_copied(&self) -> bool {
    (self.flags().0 as u32 & DagNodeFlag::Copied as u32) != 0
  }
  fn  is_unrewritable(&self) -> bool {
    (self.flags().0 as u32 & DagNodeFlag::Unrewritable as u32) != 0
  }
  fn  is_unstackable(&self) -> bool {
    (self.flags().0 as u32 & DagNodeFlag::Unstackable as u32) != 0
  }
  fn  is_ground(&self) -> bool {
    (self.flags().0 as u32 & DagNodeFlag::Ground as u32) != 0
  }
  fn  is_hash_valid(&self) -> bool {
    (self.flags().0 as u32 & DagNodeFlag::HashValid as u32) != 0
  }
  // endregion

}

// clone_trait_object!(DagNode);

impl Eq for dyn DagNode {}

impl PartialEq for dyn DagNode {
  fn eq(&self, other: &dyn DagNode) -> bool {
    // self.symbol().eq(other.symbol())
    self.symbol().get_hash_value() == other.symbol().get_hash_value()
  }
}


impl PartialOrd for dyn DagNode {
  fn partial_cmp(&self, other: &dyn DagNode) -> Option<Ordering> {
    let result = self.symbol().get_hash_value().cmp(&other.symbol().get_hash_value());
    Some(result)
  }
}

impl Ord for dyn DagNode {
  fn cmp(&self, other: &dyn DagNode) -> std::cmp::Ordering {
    self.symbol().get_hash_value().cmp(&other.symbol().get_hash_value())
  }
}
