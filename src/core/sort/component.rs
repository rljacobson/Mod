/*!

`Sort`s are a partially ordered set and thus form a lattice structure. A `ConnectedComponent` of this structure is a
set of `Sort`s that are related to each other.

*/


use crate::abstractions::{RcCell, WeakCell};
use crate::core::sort::{SortSet, WeakSort};


pub type RcConnectedComponent = RcCell<ConnectedComponent>;

#[derive(PartialEq, Eq, Default)]
pub struct ConnectedComponent {
  pub(crate) sort_count: u32,
  pub(crate) maximal_sorts_count: u32,
  error_free_flag: bool,
  sorts: SortSet,
  last_allocated_match_index: u32,
}

impl ConnectedComponent {
  // The `ConnectedComponent` takes ownership of the `Box<Sort>`.
  #[inline(always)]
  pub fn append_sort(&mut self, sort: WeakSort) -> u32 {
    let i = self.sorts.len();
    self.sorts.push(sort);
    return i as u32;
  }

  #[inline(always)]
  pub fn register_sort(&mut self) {
    self.sort_count += 1;
  }

  #[inline(always)]
  pub fn sort(&self, i: i32) -> WeakSort {
    self.sorts[i as usize].clone()
  }

  #[inline(always)]
  pub fn get_new_match_index(&mut self) -> u32 {
    self.last_allocated_match_index += 1;
    return self.last_allocated_match_index;
  }
}
