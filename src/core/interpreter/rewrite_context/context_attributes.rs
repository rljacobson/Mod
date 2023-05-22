/*!

Attributes defined for the rewriting context, implemented as a bitfield.

*/


use std::ops::{BitOr, BitOrAssign};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum ContextAttribute {
  LocalTrace = 0,
  Trace,
  TracePost,
  Abort,
  Info,
  CtrlC,
  Step,
  Interactive,
  Silent,
  DebugMode
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct ContextAttributes(u32);

impl ContextAttributes {
  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.0 == 0
  }

  #[inline(always)]
  pub fn is_set(&self, attribute: ContextAttribute) -> bool {
    self.has_attribute(attribute)
  }

  #[inline(always)]
  pub fn set(&mut self, attribute: ContextAttribute) {
    self.0 |= 1 << attribute as u32;
  }

  #[inline(always)]
  pub fn reset(&mut self, attribute: ContextAttribute) {
    self.0 &= !(1 << attribute as u32);
  }

  #[inline(always)]
  pub fn has_attribute(&self, attribute: ContextAttribute) -> bool {
    (self.0 & (1 << attribute as u32)) != 0
  }

  #[inline(always)]
  pub fn has_attributes(&self, attributes: ContextAttributes) -> bool {
    (self.0 & attributes.0) == attributes.0
  }

  #[inline(always)]
  pub fn set_trace_status(&mut self, trace_status: bool) {
    if trace_status {
      self.set(ContextAttribute::Trace);
    } else {
      self.reset(ContextAttribute::Trace);
    }
  }
}

impl BitOr for ContextAttribute {
  type Output = ContextAttributes;

  fn bitor(self, rhs: Self) -> Self::Output {
    ContextAttributes((1 << self as u32) | (1 << rhs as u32))
  }
}

impl From<ContextAttribute> for ContextAttributes {
  fn from(attribute: ContextAttribute) -> Self {
    ContextAttributes(1 << attribute as u32)
  }
}

impl BitOr for ContextAttributes {
  type Output = ContextAttributes;
  fn bitor(self, rhs: Self) -> Self::Output {
    ContextAttributes(self.0 | rhs.0)
  }
}

impl BitOr<ContextAttribute> for ContextAttributes {
  type Output = ContextAttributes;
  fn bitor(self, rhs: ContextAttribute) -> Self::Output {
    ContextAttributes(self.0 | (1 << rhs as u32))
  }
}

impl BitOrAssign<ContextAttribute> for ContextAttributes {
  fn bitor_assign(&mut self, rhs: ContextAttribute) {
    self.0 |= 1 << rhs as u32;
  }
}

