use std::ops::{BitOr, BitOrAssign};
use crate::abstractions::WeakCell;
use crate::core::{NatSet, Sort, WeakSort};
use crate::core::sort::OpDeclaration;

use super::{RcSort, RcConnectedComponent};


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum ConstructorStatus {
  // ToDo: Are the numeric values necessary?
  Constructor    = 1,
  NonConstructor = 2,
  Complex        = 1 | 2,
}

impl BitOr for ConstructorStatus {
  type Output = ConstructorStatus;

  #[inline(always)]
  fn bitor(self, rhs: Self) -> Self::Output {
    unsafe {
      std::mem::transmute(self as u8 | rhs as u8)
    }
  }
}

impl BitOrAssign for ConstructorStatus {
  #[inline(always)]
  fn bitor_assign(&mut self, rhs: Self) {
    unsafe {
      *self = std::mem::transmute(*self as u8 | rhs as u8)
    }
  }
}


#[derive(PartialEq, Eq)]
pub struct SortTable {
  nr_args                  : usize,
  op_declarations          : Vec<OpDeclaration>,
  component_vector         : Vec<RcConnectedComponent>,
  sort_diagram             : Vec<i32>,
  single_non_error_sort    : Option<WeakSort>,  // if we can only generate one non-error sort
  constructor_diagram      : Vec<i32>,
  constructor_status       : ConstructorStatus, // placeholder
  maximal_op_decl_set_table: Vec<NatSet>,       // indices of maximal op decls with range <= each sort
}

impl SortTable {
  #[inline(always)]
  pub fn arity(&self) -> usize {
    self.nr_args
  }

  #[inline(always)]
  pub fn get_maximal_op_decl_set(&mut self, target: RcSort) -> &NatSet {
    if self.maximal_op_decl_set_table.is_empty() {
      self.compute_maximal_op_decl_set_table();
    }
    &self.maximal_op_decl_set_table[target.borrow().sort_index as usize]
  }

  #[inline(always)]
  pub fn special_sort_handling(&self) -> bool {
    self.sort_diagram.is_empty()
  }

  #[inline(always)]
  pub fn add_op_declaration(&mut self, domain_and_range: Vec<RcSort>, constructor_flag: bool) {
    assert_eq!(
      domain_and_range.len() - 1,
      self.nr_args,
      "bad domain length of {} instead of {}",
      domain_and_range.len() - 1,
      self.nr_args
    );
    let nr_op_declarations = self.op_declarations.len();

    self.op_declarations.resize(nr_op_declarations + 1, OpDeclaration::default());
    self.op_declarations[nr_op_declarations] = domain_and_range.clone(); //.set_info(domain_and_range,
    // constructor_flag);
    self.constructor_status |= if constructor_flag { ConstructorStatus::Constructor } else { ConstructorStatus::NonConstructor };
  }

  #[inline(always)]
  pub fn get_op_declarations(&self) -> &Vec<OpDeclaration> {
    &self.op_declarations
  }

  #[inline(always)]
  pub fn range_component(&self) -> RcConnectedComponent {
    (&self.op_declarations[0])[self.nr_args].borrow().sort_component.clone()
  }

  #[inline(always)]
  pub fn get_range_sort(&self) -> RcSort {
    (&self.op_declarations[0])[self.nr_args].clone()
  }

  #[inline(always)]
  pub fn domain_component(&self, arg_nr: usize) -> RcConnectedComponent {
    (&self.op_declarations[0])[arg_nr].borrow().sort_component.clone()
  }

  #[inline(always)]
  pub fn get_single_non_error_sort(&self) -> Option<WeakSort> {
    self.single_non_error_sort.clone()
  }

  #[inline(always)]
  pub fn get_constructor_status(&self) -> ConstructorStatus {
    self.constructor_status
  }

  #[inline(always)]
  pub fn traverse(&self, position: usize, sort_index: usize) -> i32 {
    // ToDo: Do we need a bounds check?
    self.sort_diagram[position + sort_index]
  }

  #[inline(always)]
  pub fn constructor_traverse(&self, position: usize, sort_index: usize) -> i32 {
    // ToDo: Do we need a bounds check?
    self.constructor_diagram[position + sort_index]
  }

  pub fn domain_subsumes(&self, subsumer: usize, victim: usize) -> bool {
    let s = &self.op_declarations[subsumer];
    let v = &self.op_declarations[victim];

    for i in 0..self.nr_args {
      if !v[i].borrow().leq(s[i].as_ref()) {
        return false;
      }
    }
    true
  }

  pub fn compute_maximal_op_decl_set_table(&mut self) {
    let range           = self.range_component();
    let nr_sorts        = range.borrow().sort_count as usize;
    let nr_declarations = self.op_declarations.len();

    self.maximal_op_decl_set_table.resize(nr_sorts as usize, NatSet::new());

    for i in 0..nr_sorts {
      let target = range.borrow().sort(i);

      for j in 0..nr_declarations {
        let target_strong = target.upgrade().unwrap();
        let target_ref = &*target_strong.borrow();

        if (&self.op_declarations[j])[self.nr_args].borrow().leq(target_ref) {
          for k in 0..j {

            if self.maximal_op_decl_set_table[i].contains(k) {
              if self.domain_subsumes(k, j) {
                continue;
              } else if self.domain_subsumes(j, k) {
                self.maximal_op_decl_set_table[i].remove(k);
              }
            }

          }

          self.maximal_op_decl_set_table[i].insert(j);
        }
      }
    }
  }




}
