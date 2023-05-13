
use std::ops::BitOr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PreEquationAttribute {
  Compiled = 0, // PreEquation
  NonExecute,   // PreEquation
  Otherwise,    // Equation, "owise"
  Variant,      // Equation
  Print,        // StatementAttributeInfo--not a `PreEquation`
  Narrowing     // Rule
}
pub struct PreEquationAttributes(u8);

impl BitOr for PreEquationAttribute {
  type Output = PreEquationAttributes;

  fn bitor(self, rhs: Self) -> Self::Output {
    PreEquationAttributes(self as u8 | rhs as u8)
  }
}

impl From<PreEquationAttribute> for PreEquationAttributes {
  fn from(value: PreEquationAttribute) -> Self {
    PreEquationAttributes(1 << (value as u8))
  }
}

impl BitOr for PreEquationAttributes {
  type Output = PreEquationAttributes;
  fn bitor(self, rhs: Self) -> Self::Output {
    PreEquationAttributes(self.0 | rhs.0)
  }
}

impl BitOr<PreEquationAttribute> for PreEquationAttributes {
  type Output = PreEquationAttributes;
  fn bitor(self, rhs: PreEquationAttribute) -> Self::Output {
    PreEquationAttributes(self.0 | (1 << (rhs as u8)))
  }
}

impl PreEquationAttributes {
  pub fn has_attribute(&self, attribute: PreEquationAttribute) -> bool {
    (self.0 & attribute as u8) != 0
  }

  pub fn has_attributes(&self, attributes: PreEquationAttributes) -> bool {
    (self.0 & attributes.0) == attributes.0
  }
}
