use std::{
  fmt::{write, Display, Formatter},
  ops::{BitOr, BitOrAssign},
};

use crate::core::format::{FormatStyle, Formattable};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PreEquationAttribute {
  Compiled = 0, // PreEquation
  NonExecute,   // PreEquation
  Otherwise,    // Equation, "owise"
  Variant,      // Equation
  Print,        // StatementAttributeInfo--not a `PreEquation`
  Narrowing,    // Rule
  Bad,          // A malformed pre-equation
}

impl Display for PreEquationAttribute {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PreEquationAttribute::Compiled => {
        write!(f, "Compiled")
      }

      PreEquationAttribute::NonExecute => {
        write!(f, "NonExecute")
      }

      PreEquationAttribute::Otherwise => {
        write!(f, "Otherwise")
      }

      PreEquationAttribute::Variant => {
        write!(f, "Variant")
      }

      PreEquationAttribute::Print => {
        write!(f, "Print")
      }

      PreEquationAttribute::Narrowing => {
        write!(f, "Narrowing")
      }

      PreEquationAttribute::Bad => {
        write!(f, "Bad")
      }
    }
  }
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct PreEquationAttributes(u8);

impl PreEquationAttributes {
  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.0 == 0
  }

  #[inline(always)]
  pub fn set(&mut self, attr: PreEquationAttribute) {
    *self |= attr;
  }

  #[inline(always)]
  pub fn reset(&mut self, attr: PreEquationAttribute) {
    self.0 &= !(1 << attr as u8);
  }
}

impl BitOr for PreEquationAttribute {
  type Output = PreEquationAttributes;

  #[inline(always)]
  fn bitor(self, rhs: Self) -> Self::Output {
    PreEquationAttributes(1 << self as u8 | 1 << rhs as u8)
  }
}

impl From<PreEquationAttribute> for PreEquationAttributes {
  #[inline(always)]
  fn from(value: PreEquationAttribute) -> Self {
    PreEquationAttributes(1 << (value as u8))
  }
}

impl BitOr for PreEquationAttributes {
  type Output = PreEquationAttributes;

  #[inline(always)]
  fn bitor(self, rhs: Self) -> Self::Output {
    PreEquationAttributes(self.0 | rhs.0)
  }
}

impl BitOr<PreEquationAttribute> for PreEquationAttributes {
  type Output = PreEquationAttributes;

  #[inline(always)]
  fn bitor(self, rhs: PreEquationAttribute) -> Self::Output {
    PreEquationAttributes(self.0 | (1 << (rhs as u8)))
  }
}

impl BitOrAssign<PreEquationAttribute> for PreEquationAttributes {
  #[inline(always)]
  fn bitor_assign(&mut self, rhs: PreEquationAttribute) {
    self.0 |= 1 << (rhs as u8);
  }
}

impl PreEquationAttributes {
  #[inline(always)]
  pub fn has_attribute(&self, attribute: PreEquationAttribute) -> bool {
    (self.0 & attribute as u8) != 0
  }

  #[inline(always)]
  pub fn has_attributes(&self, attributes: PreEquationAttributes) -> bool {
    (self.0 & attributes.0) == attributes.0
  }
}

impl Formattable for PreEquationAttributes {
  fn repr(&self, _style: FormatStyle) -> String {
    if self.0 == 0 {
      return "".to_string();
    }

    let mut accumulator = " [".to_string();

    let mut space = "";
    for i in 0u8..6u8 {
      let attribute: PreEquationAttribute = unsafe { std::mem::transmute(i) };
      if self.has_attribute(attribute) {
        accumulator.push_str(space);
        accumulator.push_str(attribute.to_string().as_str());
        space = " ";
      }
    }
    accumulator.push_str("]");
    accumulator
  }
}
