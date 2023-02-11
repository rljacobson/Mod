/*!

Items related to sorts (types).

*/

use std::cmp::Ordering;
use std::fmt::Display;
use std::mem::size_of;
use reffers::rc1::{Strong, Weak};
use crate::NatSet;

#[repr(i32)]
pub enum SpecialSorts
{
  Kind = 0,
  // ErrorSort = 0,
  FirstUserSort = 1,
  SortUnknown = -1
}

type ErrorSort = SpecialSorts::Kind;

pub type RcSort = Strong<Sort>;
// The pointers inside a sort to other sorts have to be weak pointers, because we expect there to be cycles.
pub type RcWeakSort = Weak<Sort>;

pub struct Sort {
  name      : u32, // a.k.a ID
  sort_index: u32, // Used as `number_unresolved_supersorts` when computing supersorts.
  fast_test : u32,
  subsorts  : Vec<RcWeakSort>,
  supersorts: Vec<RcWeakSort>,
  leq_sorts : NatSet,

  // Todo: Should this be an Option<..>?
  sort_component: ConnectedComponent
}

impl Sort {
  /// The idea is that it's faster to avoid calling self.leq_sorts.contains(),
  /// but only returns the correctresult if (fastTest - 1) <= NatSet::smallIntBound.
  // Todo: This probably does not give a speed advantage. Benchmark.
  pub fn fast_geq(&self, index: u32) -> bool {
    if index >= self.fast_test {
      true
    } else {
      self.leq_sorts.contains(index as usize)
    }
  }

  /// See `fast_geq(..)`.
  pub fn fast_geq_sufficient(&self) {
    // We assume a usize, which is 64 bits on most workstations.
    // Todo: This is another reason to get rid of this optimization. Creates platform dependence.
    (self.fast_test - 1) <= 8*size_of::<usize>() as u32; //NatSet::smallIntBound
  }

  /// Computes self <= other.
  pub fn leq(&self, other: &Sort) -> bool {
    other.leq_sorts.contains(self.sort_index as usize)
  }

  /// Computes self <= other_sort where other_sort is the sort associated to index.
  pub fn leq_index(&self, index: u32) -> bool {
    self.sort_component.sort(index as usize).get_ref().leq_sorts.contains(self.sort_index as usize)
  }

}

impl Display for Sort {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

    // If let Some(c) = &self.sort_component
    let c = &self.sort_component;
    if self.sort_index == SpecialSorts::Kind as u32 {
      let sort_list = (1..c.nr_maximal_sorts).map(
                |idx| c.sort(idx).to_string()
              ).collect().join(", ");

      write!(f, "[{}]", sort_list)
    } else {
      // Todo: Fix this when a symbol table exists.
      write!(f, "{}", "Token::sortName(sort->id())")
    }

  }
}

#[derive(Eq, PartialEq)]
pub(crate) struct ConnectedComponent {
  sort_count                : u32,
  maximal_sorts_count       : u32,
  error_free_flag           : bool,
  sorts                     : Vec<RcWeakSort>,
  last_allocated_match_index: u32
}

impl ConnectedComponent {
  // The `ConnectedComponent` takes ownership of the `Box<Sort>`.
  pub fn append_sort(&mut self, sort: RcWeakSort) -> u32 {
    let  i = self.sorts.len();
    self.sorts.push(sort);
    return i as u32;
  }

  pub fn register_sort(&mut self) {
    self.sort_count += 1;
  }

  pub fn sort(&self, i: usize)-> RcWeakSort {
    self.sorts[i].clone()
  }

  pub fn get_new_match_index(&mut self)-> u32 {
    self.last_allocated_match_index +=  1;
    return self.last_allocated_match_index
  }

}

#[inline(always)]
pub fn index_leq_sort(index: u32, sort: &Sort) -> bool {
  assert_ne!(index, SpecialSorts::SortUnknown as u32, "unknown sort");
  if index >= sort.fast_test {
    return true;
  }
  return sort.leq_sorts.contains(index as usize);
}

#[inline(always)]
pub fn sort_leq_index(sort: &Sort, index: u32) -> bool {
  index_leq_sort(sort.sortIndex, sort.sort_component.sort(index as usize).get_ref().as_ref())
}