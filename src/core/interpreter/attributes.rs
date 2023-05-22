/*!

Attributes (flags) for interpreters implemented as a bit field.


*/

use std::ops::{BitOr, BitOrAssign};

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum InterpreterAttribute {
  // Show (information) flags
  ShowCommand   = 0x1,
  ShowStats     = 0x2,
  ShowTiming    = 0x4,
  ShowBreakdown = 0x8,

  // Loop mode flags
  ShowLoopStats    = 0x10,
  ShowLoopTiming   = 0x20,
  ERewriteLoopMode = 0x40,

  // Memoization flags
  AutoClearMemo = 0x100,

  // Profiler flags
  Profile          = 0x200,
  AutoClearProfile = 0x400,

  // Debugger flags
  Break = 0x800,

  // Tracer flags
  Trace             = 0x1000,
  TraceCondition    = 0x2000,
  TraceWhole        = 0x4000,
  TraceSubstitution = 0x8000,
  TraceSelect       = 0x10000,
  TraceMb           = 0x20000,  // Membership
  TraceEq           = 0x40000,  // Equation
  TraceRl           = 0x80000,  // Rule
  TraceSd           = 0x100000, // Sort
  TraceRewrite      = 0x200000,
  TraceBody         = 0x400000,
  TraceBuiltin      = 0x800000,

  // Unimplemented Print attribute flags
  /*
  PrintAttribute        = 0x1000000,
  PrintAttributeNewline = 0x2000000,
  */

  // Counter flags
  AutoClearRules = 0x40000000,

  // Compiler flags
  CompileCount = 0x80000000,

  // Composite flags

  ExceptionFlags = InterpreterAttribute::Trace as u32
      | InterpreterAttribute::Break as u32
      | InterpreterAttribute::Profile as u32 ,
      // | InterpreterAttribute::PrintAttribute as u32, // Not implemented

  #[default]
  DefaultFlags = InterpreterAttribute::ShowCommand as u32
      | InterpreterAttribute::ShowStats as u32
      | InterpreterAttribute::ShowTiming as u32
      | InterpreterAttribute::ShowLoopTiming as u32
      | InterpreterAttribute::CompileCount as u32
      | InterpreterAttribute::TraceCondition as u32
      | InterpreterAttribute::TraceSubstitution as u32
      | InterpreterAttribute::TraceMb as u32
      | InterpreterAttribute::TraceEq as u32
      | InterpreterAttribute::TraceRl as u32
      | InterpreterAttribute::TraceSd as u32
      | InterpreterAttribute::TraceRewrite as u32
      | InterpreterAttribute::TraceBody as u32
      | InterpreterAttribute::TraceBuiltin as u32
      | InterpreterAttribute::AutoClearProfile as u32
      | InterpreterAttribute::AutoClearRules as u32
      // | InterpreterAttribute::PrintAttributeNewline as u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct InterpreterAttributes(u32);

impl Default for InterpreterAttributes {
  fn default() -> Self {
    InterpreterAttribute::DefaultFlags.into()
  }
}

impl InterpreterAttributes {
  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.0 == 0
  }

  #[inline(always)]
  pub fn is_set(&self, attribute: InterpreterAttribute) -> bool {
    self.has_attribute(attribute)
  }

  #[inline(always)]
  pub fn set(&mut self, attribute: InterpreterAttribute) {
    self.0 |= attribute as u32
  }

  #[inline(always)]
  pub fn reset(&mut self, attribute: InterpreterAttribute) {
    self.0 &= !(attribute as u32)
  }

  #[inline(always)]
  pub fn has_attribute(&self, attribute: InterpreterAttribute) -> bool {
    (self.0 & attribute as u32) != 0
  }

  #[inline(always)]
  pub fn has_attributes(&self, attributes: InterpreterAttributes) -> bool {
    (self.0 & attributes.0) == attributes.0
  }
}

impl BitOr for InterpreterAttribute {
  type Output = InterpreterAttributes;

  fn bitor(self, rhs: Self) -> Self::Output {
    InterpreterAttributes(self as u32 | rhs as u32)
  }
}

impl From<InterpreterAttribute> for InterpreterAttributes {
  fn from(value: InterpreterAttribute) -> Self {
    InterpreterAttributes(value as u32)
  }
}

impl BitOr for InterpreterAttributes {
  type Output = InterpreterAttributes;
  fn bitor(self, rhs: Self) -> Self::Output {
    InterpreterAttributes(self.0 | rhs.0)
  }
}

impl BitOr<InterpreterAttribute> for InterpreterAttributes {
  type Output = InterpreterAttributes;
  fn bitor(self, rhs: InterpreterAttribute) -> Self::Output {
    InterpreterAttributes(self.0 | rhs as u32)
  }
}

impl BitOrAssign<InterpreterAttribute> for InterpreterAttributes {
  fn bitor_assign(&mut self, rhs: InterpreterAttribute) {
    self.0 |= rhs as u32;
  }
}



