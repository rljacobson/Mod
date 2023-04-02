/*!

Items related to sorts (types).

*/

use std::fmt::Display;
use std::mem::size_of;
use reffers::rc1::{Strong, Weak};

use crate::core::NatSet;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
pub enum SpecialSort
{
  Kind = 0,
  // ErrorSort = 0,
  FirstUserSort = 1,
  Unknown = -1
}

impl SpecialSort{
  // This is how you have an alias of an existing variant.
  //    `SpecialSort::Kind==SpecialSort::ErrorSort`
  #[allow(non_upper_case_globals)]
  pub const ErrorSort: SpecialSort = SpecialSort::Kind;
}

pub type RcSort = Strong<Sort>;
// The pointers inside a sort to other sorts have to be weak pointers, because we expect there to be cycles.
pub type RcWeakSort = Weak<Sort>;
/// A lot of things have their own list of sorts.
pub type SortSet = Vec<RcWeakSort>;


#[derive(Clone, Default)]
pub struct Sort {
  name      : u32, // a.k.a ID
  sort_index: i32, // Used as `number_unresolved_supersorts` when computing supersorts.
  fast_test : i32,
  subsorts  : SortSet,
  supersorts: SortSet,
  leq_sorts : NatSet,

  // Todo: Should this be an Option<..>?
  sort_component: RcStrongConnectedComponent
}

impl Sort {
  /// The idea is that it's faster to avoid calling self.leq_sorts.contains(),
  /// but only returns the correctresult if (fastTest - 1) <= NatSet::smallIntBound.
  // Todo: This probably does not give a speed advantage. Benchmark.
  pub fn fast_geq(&self, index: i32) -> bool {
    if index >= self.fast_test {
      true
    } else {
      self.leq_sorts.contains(index as usize)
    }
  }

  /// See `fast_geq(..)`.
  pub fn fast_geq_sufficient(&self) -> bool {
    // We assume a usize, which is 64 bits on most workstations.
    // Todo: This is another reason to get rid of this optimization. Creates platform dependence.
    (self.fast_test - 1) <= 8*size_of::<usize>() as i32 //NatSet::smallIntBound
  }

  /// Computes self <= other.
  pub fn leq(&self, other: &Sort) -> bool {
    other.leq_sorts.contains(self.sort_index as usize)
  }

  /// Computes self <= other_sort where other_sort is the sort associated to index.
  pub fn leq_index(&self, index: u32) -> bool {
    self.sort_component.get_ref().sort(index as usize).get_ref().leq_sorts.contains(self.sort_index as usize)
  }

}

impl Display for Sort {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    // If let Some(c) = &self.sort_component
    let c = self.sort_component.get_ref();
    if self.sort_index == SpecialSort::Kind as i32 {
      let sort_list = (1..c.maximal_sorts_count).map(
                |idx| c.sort(idx as usize).get_ref().to_string()
              ).collect::<Vec<String>>().join(", ");

      write!(f, "[{}]", sort_list)
    } else {
      // Todo: Fix this when a symbol table exists.
      write!(f, "{}", "Token::sortName(sort->id())")
    }

  }
}

impl PartialEq for Sort {
  fn eq(&self, other: &Sort) -> bool {
    return self.name == other.name
          && self.sort_index == other.sort_index;
    /*
    if self.name == other.name
        && self.sort_index == other.sort_index
        && self.fast_test == other.fast_test
        && self.leq_sorts == other.leq_sorts
        && self.sort_component == other.sort_component
        && self.subsorts.len()==other.subsorts.len()
        && self.supersorts.len() == other.supersorts.len()
    {

      for (s, t) in self.subsorts.iter().zip(other.subsorts.iter()) {
        match s.try_get_ref() {
            Ok(sr) => {
              match t.try_get_ref() {
                Ok(tr) => {
                  if sr!=tr {
                    return false;
                  }
                  /* continue below */
                },
                Err(_) => return false,
            }
            },
            Err(_) => return false,
        }
      }
      //  self.supersorts == other.supersorts
      for (s, t) in self.supersorts.iter().zip(other.supersorts.iter()) {
        match s.try_get_ref() {
            Ok(sr) => {
              match t.try_get_ref() {
                Ok(tr) => {
                  /* continue below */
                  if sr!=tr {
                    return false;
                  }
                },
                Err(_) => return false,
            }
            },
            Err(_) => return false,
        }
      }
      true
    } else {
      false
    }
    */
  }
}


impl PartialEq<SpecialSort> for Sort {
    fn eq(&self, other: &SpecialSort) -> bool {
        self.sort_index == *other as i32
    }
}

impl PartialEq<Sort> for SpecialSort {
    fn eq(&self, other: &Sort) -> bool {
        other.sort_index == *self as i32
    }
}

impl Eq for Sort {}


type RcStrongConnectedComponent = Strong<ConnectedComponent>;


#[derive(Default)]
pub(crate) struct ConnectedComponent {
  sort_count                : u32,
  maximal_sorts_count       : u32,
  error_free_flag            : bool,
  sorts                     : SortSet,
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
pub fn index_leq_sort(index: i32, sort: &Sort) -> bool {
  assert_ne!(index, SpecialSort::Unknown as i32, "unknown sort");
  if index >= sort.fast_test {
    return true;
  }
  return sort.leq_sorts.contains(index as usize);
}

#[inline(always)]
pub fn sort_leq_index(sort: &Sort, index: i32) -> bool {
  index_leq_sort(sort.sort_index, sort.sort_component.get_ref().sort(index as usize).get_ref().as_ref())
}
