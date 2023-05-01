/*!

Trait for DAG nodes.

*/

use std::collections::HashSet;
use std::{any::Any, fmt::Display};
use std::cmp::Ordering;
use std::ptr::addr_of;
use std::rc::Rc;

// use dyn_clone::{clone_trait_object, DynClone};
use shared_vector::{AtomicSharedVector, SharedVector};

use crate::{
  abstractions::{BigInteger, RcCell},
  core::{RcSort, Sort, SpecialSort},
  theory::{MaybeSubproblem, Outcome, Symbol}
};
use crate::core::Substitution;
use crate::theory::free_theory::RcFreeSymbol;
use crate::theory::{DagNodeFlags, ExtensionInfo, RcTerm, Subproblem};

use super::{RcSymbol, SymbolType};

// pub type BcDagNode = Box<Cell<DagNode>>;
pub type BcDagNode = Box<dyn DagNode>;
pub type RcDagNode = RcCell<dyn DagNode>;
pub type NodeList = SharedVector<RcDagNode>;
pub type AtomicNodeList = AtomicSharedVector<RcDagNode>;

/// This struct owns the DagNode. If we just want a reference, we use a tuple `(dag_node.as_ref(), multiplicity)`.
#[derive(Clone)]
pub struct DagPair {
  pub(crate) dag_node: RcDagNode,
  pub(crate) multiplicity: u32,
}

pub struct DagNodeMembers {
  pub(crate) top_symbol: RcSymbol,
  pub(crate) args      : NodeList,
  // pub(crate) sort      : Option<RcSort>,
  pub(crate) flags     : DagNodeFlags,
  pub(crate) sort_index: i32,
}

// Todo: Maude puts `copyPointer` and `top_symbol` in a union for optimization.
pub trait DagNode {

  // region Member Getters and Setters

  /// Trait level access to members for shared implementation
  fn dag_node_members(&self) -> &DagNodeMembers;
  fn dag_node_members_mut(&mut self) -> &mut DagNodeMembers;

  /// Returns an iterator over `(RcDagNode, u32)` pairs for the arguments.
  #[inline(always)]
  fn iter_args(&self) -> Box<dyn Iterator<Item=RcDagNode> + '_> {
    Box::new(self.dag_node_members().args.iter().cloned()) //.map(|pair| (pair.dag_node.clone(), pair.multiplicity)))
  }

  /// Gives the top symbol of this term.
  #[inline(always)]
  fn symbol(&self) -> RcSymbol {
    self.dag_node_members().top_symbol.clone()
  }

  // Todo: Is this needed?
  #[inline(always)]
  fn symbol_mut(&mut self) -> &mut dyn Symbol {
    Rc::get_mut(&mut self.dag_node_members_mut().top_symbol).unwrap()
  }

  #[inline(always)]
  fn get_sort(&self) -> Option<RcSort> {
    let sort_index: i32 = self.get_sort_index();
    match sort_index {

      n if n == SpecialSort::Unknown as i32 => {
        None
      }

      // Anything else
      sort_index => {
        self.dag_node_members()
            .top_symbol
            .sort_table()
            .range_component()
            .borrow()
            .sort(sort_index)
            .upgrade()
      }
    }
  }

  #[inline(always)]
  fn set_sort_index(&mut self, sort_index: i32) {
    self.dag_node_members_mut().sort_index = sort_index;
  }

  #[inline(always)]
  fn get_sort_index(&self) -> i32 {
    self.dag_node_members().sort_index
  }

  /// The number of arguments
  #[inline(always)]
  fn len(&self) -> usize {
    self.dag_node_members().args.len()
  }

  #[inline(always)]
  fn flags(&self) -> DagNodeFlags{
    self.dag_node_members().flags
  }

  #[inline(always)]
  fn set_flags(&mut self, flags: DagNodeFlags) {
    self.dag_node_members_mut().flags.0 |= flags.0;
  }

  // endregion


  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  // fn as_ptr(&self) -> *const dyn DagNode;

  fn as_ptr(&self) -> *const dyn DagNode;

  /// Defines a partial order on `DagNode`s. Unlike the `Ord`/`PartialOrd` implementation, this method also compares
  /// the arguments.
  fn compare(&self, other: &dyn DagNode) -> Ordering {
    // let symbol_order = self.cmp(other);
    let s = self.symbol();
    let symbol_order = //Ord::cmp(s, other.symbol());
    s.get_hash_value().cmp(&other.symbol().get_hash_value());

    match symbol_order {
      Ordering::Equal => self.compare_arguments(other),
      _ => symbol_order,
    }
  }

  fn compare_arguments(&self, other: &dyn DagNode) -> Ordering;

  #[inline(always)]
  fn leq_sort(&self, sort: &Sort) -> bool {
    assert_ne!(self.get_sort_index(), SpecialSort::Unknown as i32, "unknown sort");
    self.get_sort().unwrap().as_ref().leq(sort)
  }

  /// Sets the sort_index of self. This is a method on Symbol in Maude.
  fn compute_base_sort(&mut self) -> i32;


  fn check_sort(&mut self, bound_sort: RcSort) -> (Outcome, MaybeSubproblem) {
    if self.get_sort().is_some() {
      return (self.leq_sort(bound_sort.as_ref()).into(), None);
    }

    // This is a weird code smell.
    // self.symbol_mut().compute_base_sort(self);
    // The ACUSymbol just turns around and calls `compute_base_sort` on the owning `DagNode`.
    // It should be a method of DagNode which sets the DagNode's sort index. So that's what we do.
    self.compute_base_sort();

    if self.leq_sort(bound_sort.as_ref()) {
      if !self.symbol().sort_constraint_free() {
        self.set_sort_index(SpecialSort::Unknown as i32);
      }
    } else {
      if self.symbol().sort_constraint_free() {
        return (Outcome::Failure, None);
      }
      self.set_sort_index(SpecialSort::Unknown as i32);
      // Todo: Implement `SortCheckSubproblem`.
      // let returned_subproblem = SortCheckSubproblem::new(this, bound_sort);
      // return (Outcome::Success, Some(returned_subproblem))
    }

    return (Outcome::Success, None);
  }

  // region Flag Manipulation
  #[inline(always)]
  fn is_reduced(&self) -> bool {
    self.flags().is_reduced()
  }
  #[inline(always)]
  fn is_copied(&self) -> bool {
    self.flags().is_copied()
  }
  #[inline(always)]
  fn is_unrewritable(&self) -> bool {
    self.flags().is_unrewritable()
  }
  #[inline(always)]
  fn is_unstackable(&self) -> bool {
    self.flags().is_unstackable()
  }
  #[inline(always)]
  fn is_ground(&self) -> bool {
    self.flags().is_ground()
  }
  #[inline(always)]
  fn is_hash_valid(&self) -> bool {
    self.flags().is_hash_valid()
  }
  // endregion

  fn termify(&self) -> RcTerm;

  fn shallow_copy(&self) -> RcDagNode;

  // In Maude this is a method on DagNode, but it makes more sense as a method on `LHSAutomaton`.
  // fn match_variable(…)

}

// clone_trait_object!(DagNode);

// region PartialEq, Eq, PartialOrd, Ord implementations
impl Eq for dyn DagNode {}

impl PartialEq for dyn DagNode {
  #[inline(always)]
  fn eq(&self, other: &dyn DagNode) -> bool {
    // self.symbol().eq(other.symbol())
    self.symbol().get_hash_value() == other.symbol().get_hash_value()
  }
}

impl PartialOrd for dyn DagNode {
  #[inline(always)]
  fn partial_cmp(&self, other: &dyn DagNode) -> Option<Ordering> {
    let result = self
    .symbol()
    .get_hash_value()
    .cmp(&other.symbol().get_hash_value());
    Some(result)
  }
}

impl Ord for dyn DagNode {
  #[inline(always)]
  fn cmp(&self, other: &dyn DagNode) -> Ordering {
    self.symbol()
    .get_hash_value()
    .cmp(&other.symbol().get_hash_value())
  }
}
// endregion

impl Display for dyn DagNode {
  /*
  It's not clear what this method should do. In Maude, the MixfixModule::graphPrint method is in charge of printing
  the graph. It produces output as in the following example session.

  Maude> red in PEANO : (s (o)) + (s(s (o)))  .
  reduce in PEANO : Begin{Graph Representation}
  [Term has 6 operator symbols while graph has 4 nodes.]
  #0 = _+_(#1, #3)
  #1 = s_(#2)
  #2 = o
  #3 = s_(#1)
  End{Graph Representation} .
  rewrites: 2 in 0ms cpu (0ms real) (2000000 rewrites/second)
  result NzNat: Begin{Graph Representation}
  [Term has 4 operator symbols while graph has 4 nodes.]
  #0 = s_(#1)
  #1 = s_(#2)
  #2 = s_(#3)
  #3 = o
  End{Graph Representation}

  However, that method seems to include code that should belong to subclasses. Also, it's not clear why it lives in
  the MixfixModule class. For now, we put the graphPrint code here and call a trait method `repr` that has a default
  implementation for the subclass-specific stuff.
  */

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let dag_node = self;
    let mut visited: HashSet<*const dyn DagNode> = HashSet::new();
    let mut counts: Vec<BigInteger> = vec![];

    graph_count(dag_node, &mut visited, &mut counts);

    let operators = &counts[0];
    let nodes = visited.len();

    writeln!(f, "Begin{{Graph Representation}}")?;
    writeln!(
      f,
      "[Term has {} operator symbol{} while graph has {} node{}.]",
      operators,
      if *operators == 1 { "" } else { "s" },
      nodes,
      if nodes == 1 { "" } else { "s" },
    )?;

    for (i, &dag_node_ptr) in visited.iter().enumerate() {
      let dag_node = unsafe{&*dag_node_ptr};
      let symbol = dag_node.symbol();

      write!(f, "#{} = {}", i, symbol.name())?;
      write!(f, "(")?;

      let mut first = true;
      for a in dag_node.iter_args() {
        if !first { write!(f, ", ")?; }
        write!(
          f,
          "#{}",
          visited.iter()
                 .position(
                    |&x| x == addr_of!(*a.borrow())
                 ).unwrap()
        )?;
        first = false;
      }
      write!(f, ")")?;

    }

    Ok(())

  }

}


fn graph_count(
  dag_node: &dyn DagNode,
  visited: &mut HashSet<*const dyn DagNode>,
  counts: &mut Vec<BigInteger>
)
{
  let dag_node_ptr: *const dyn DagNode = dag_node.as_ptr();
  visited.insert(dag_node_ptr);

  let index = counts.len();
  assert_eq!(index, visited.iter().position(|&x| x == dag_node_ptr).unwrap(), "counts out of step");
  counts.push(0);

  let mut count: BigInteger = 1;

  for d in dag_node.iter_args().map(|v| v.clone()) {
    let d_ptr = d.as_ptr();
    if !visited.contains(&d_ptr.cast_const()) {
      graph_count(d.as_ref(), visited, counts);
    }

    let child_count = counts[visited.iter().position(|&x| x == d_ptr).unwrap()];
    assert_ne!(child_count, 0, "cycle in dag");
    count += child_count;
  }
  counts[index] = count;
}
