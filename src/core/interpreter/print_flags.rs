/*!

Flags that determine how to print things for interpreters.

*/

use std::ops::{BitOr, BitOrAssign};

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum PrintFlag {
  PrintGraph = 0x1,
  PrintConceal = 0x2,
  PrintFormat = 0x4,
  PrintMixfix = 0x8,
  PrintWithParens = 0x10,
  PrintColor = 0x20,
  PrintDisambigConst = 0x40,
  PrintWithAliases = 0x100,
  PrintFlat = 0x200,
  PrintNumber = 0x400,
  PrintRat = 0x800,

  #[default]
  DefaultPrintFlags = PrintFlag::PrintFormat as u32
      | PrintFlag::PrintMixfix as u32
      | PrintFlag::PrintWithAliases as u32
      | PrintFlag::PrintFlat as u32
      | PrintFlag::PrintNumber as u32
      | PrintFlag::PrintRat as u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PrintFlags(u32);

impl Default for PrintFlags {
  fn default() -> Self {
    PrintFlag::DefaultPrintFlags.into()
  }
}

impl PrintFlags {
  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.0 == 0
  }

  #[inline(always)]
  pub fn is_set(&self, attribute: PrintFlag) -> bool {
    self.has_attribute(attribute)
  }

  #[inline(always)]
  pub fn set(&mut self, attribute: PrintFlag) {
    self.0 |= attribute as u32
  }

  #[inline(always)]
  pub fn reset(&mut self, attribute: PrintFlag) {
    self.0 &= !(attribute as u32)
  }

  #[inline(always)]
  pub fn has_attribute(&self, attribute: PrintFlag) -> bool {
    (self.0 & attribute as u32) != 0
  }

  #[inline(always)]
  pub fn has_attributes(&self, attributes: PrintFlags) -> bool {
    (self.0 & attributes.0) == attributes.0
  }
}

impl BitOr for PrintFlag {
  type Output = PrintFlags;

  fn bitor(self, rhs: Self) -> Self::Output {
    PrintFlags(self as u32 | rhs as u32)
  }
}

impl From<PrintFlag> for PrintFlags {
  fn from(value: PrintFlag) -> Self {
    PrintFlags(value as u32)
  }
}

impl BitOr for PrintFlags {
  type Output = PrintFlags;
  fn bitor(self, rhs: Self) -> Self::Output {
    PrintFlags(self.0 | rhs.0)
  }
}

impl BitOr<PrintFlag> for PrintFlags {
  type Output = PrintFlags;
  fn bitor(self, rhs: PrintFlag) -> Self::Output {
    PrintFlags(self.0 | rhs as u32)
  }
}

impl BitOrAssign<PrintFlag> for PrintFlags {
  fn bitor_assign(&mut self, rhs: PrintFlag) {
    self.0 |= rhs as u32;
  }
}



