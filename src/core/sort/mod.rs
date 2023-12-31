/*!

Items related to sorts (types).

*/

mod component;
mod sort_table;

use std::{fmt::Display, mem::size_of};

pub use component::{ConnectedComponent, RcConnectedComponent};
pub use sort_table::SortTable;

use crate::abstractions::{IString, NatSet, RcCell, WeakCell};


pub type RcSort = RcCell<Sort>;
// The pointers inside a sort to other sorts have to be weak pointers, because we expect there to be cycles.
pub type WeakSort = WeakCell<Sort>;
/// A lot of things have their own list of sorts. Weak pointers are used to break the cycles.
pub type SortSet = Vec<WeakSort>;
pub type OpDeclaration = Vec<RcSort>;


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum SpecialSort {
  Kind          = 0,
  // ErrorSort     = 0,
  FirstUserSort = 1,
  Unknown       = -1,
}

impl SpecialSort {
  // This is how you have an alias of an existing variant.
  //    `SpecialSort::Kind==SpecialSort::ErrorSort`
  #[allow(non_upper_case_globals)]
  pub const ErrorSort: SpecialSort = SpecialSort::Kind;
}


#[derive(Clone, Default)]
pub struct Sort {
  pub name:       IString, // a.k.a ID
  pub sort_index: i32,     // Used as `number_unresolved_supersorts` when computing supersorts.
  pub fast_test:  i32,
  pub subsorts:   SortSet,
  pub supersorts: SortSet,
  pub leq_sorts:  NatSet,

  /// The connected component this sort belongs to.
  /// The ConnectedComponent holds weak references.
  // Todo: Should this be an Option<..>?
  pub sort_component: RcConnectedComponent,
}

impl Sort {
  pub fn error_free_maximal(&self) -> bool {
    let component = self.sort_component.borrow();
    self.sort_index == 1 && component.maximal_sorts_count == 1 && component.error_free
  }

  /// The idea is that it's faster to avoid calling `self.leq_sorts.contains()`,
  /// but only returns the correct result if `(fastTest - 1) <= NatSet::smallIntBound`.
  // Todo: This probably does not give a speed advantage. Benchmark.
  #[inline(always)]
  pub fn fast_geq(&self, index: i32) -> bool {
    if index >= self.fast_test {
      true
    } else {
      self.leq_sorts.contains(index as usize)
    }
  }

  /// See `fast_geq(..)`.
  #[inline(always)]
  pub fn fast_geq_sufficient(&self) -> bool {
    // We assume a usize, which is 64 bits on most workstations.
    // Todo: This is another reason to get rid of this optimization. Creates platform dependence.
    (self.fast_test - 1) <= 8 * size_of::<usize>() as i32 //NatSet::smallIntBound
  }

  /// Computes self <= other.
  #[inline(always)]
  pub fn leq(&self, other: &Sort) -> bool {
    other.leq_sorts.contains(self.sort_index as usize)
  }

  /// Computes self <= other_sort where other_sort is the sort associated to index.
  #[inline(always)]
  pub fn leq_index(&self, index: u32) -> bool {
    self
      .sort_component
      .as_ref()
      .sort(index.try_into().unwrap())
      .upgrade()
      .unwrap()
      .as_ref()
      .leq_sorts
      .contains(self.sort_index as usize)
  }
}

// region trait impls

impl Display for Sort {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    // If let Some(c) = &self.sort_component
    let c = self.sort_component.borrow();
    if self.sort_index == SpecialSort::Kind as i32 {
      let sort_list = (1..c.maximal_sorts_count)
        .map(|idx| c.sort(idx.try_into().unwrap()).upgrade().unwrap().as_ref().to_string())
        .collect::<Vec<String>>()
        .join(", ");

      write!(f, "[{}]", sort_list)
    } else {
      write!(f, "{}", self.name)
    }
  }
}

impl PartialEq for Sort {
  fn eq(&self, other: &Sort) -> bool {
    return self.name == other.name && self.sort_index == other.sort_index;
  }
}

impl PartialEq<SpecialSort> for Sort {
  #[inline(always)]
  fn eq(&self, other: &SpecialSort) -> bool {
    self.sort_index == *other as i32
  }
}

impl PartialEq<Sort> for SpecialSort {
  #[inline(always)]
  fn eq(&self, other: &Sort) -> bool {
    other.sort_index == *self as i32
  }
}

impl Eq for Sort {}

// endregion


#[inline(always)]
pub fn index_leq_sort(index: i32, sort: &Sort) -> bool {
  assert_ne!(index, SpecialSort::Unknown as i32, "unknown sort");
  if index >= sort.fast_test {
    return true;
  }
  return sort.leq_sorts.contains(index as usize);
}

#[inline(always)]
pub fn sort_leq_index(sort: &Sort, index: i32) -> bool {
  index_leq_sort(
    sort.sort_index,
    sort
      .sort_component
      .as_ref()
      .sort(index.try_into().unwrap())
      .upgrade()
      .unwrap()
      .as_ref(),
  )
}

// Equality is implemented in WeakCell as pointer equality.
// impl PartialEq for WeakSort
// impl Eq for WeakSort
