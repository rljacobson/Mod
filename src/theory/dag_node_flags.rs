#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum DagNodeFlag {
  Reduced      = 1,  // Reduced up to strategy by equations
  Copied       = 2,  // Copied in current copy operation; copyPointer valid
  Unrewritable = 4,  // Reduced and not rewritable by rules
  Unstackable  = 8,  // Unrewritable and all subterms unstackable or frozen
  Ground       = 16, // No variables occur below this node
  HashValid    = 32, // Node has a valid hash value (storage is theory dependent)
}

impl DagNodeFlag {
  // We can share the same bit as UNREWRITABLE for this flag since the rule rewriting strategy that needs UNREWRITABLE
  // never be combined with variant narrowing. Implemented as associated type since Rust does not allow variant aliases.
  //    IRREDUCIBLE_BY_VARIANT_EQUATIONS = 4
  #[allow(non_upper_case_globals)]
  pub const IrreducibleByVariantEquations: DagNodeFlag = DagNodeFlag::Unrewritable;
}

impl std::ops::BitOr<DagNodeFlag> for DagNodeFlags {
  type Output = Self;

  fn bitor(self, rhs: DagNodeFlag) -> Self::Output {
    DagNodeFlags(self.0 | rhs as u32)
  }
}

impl std::ops::BitOr for DagNodeFlags {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    DagNodeFlags(self.0 | rhs.0)
  }
}

impl std::ops::BitAnd for DagNodeFlags {
  type Output = Self;

  fn bitand(self, rhs: Self) -> Self::Output {
    DagNodeFlags(self.0 & rhs.0)
  }
}

impl From<DagNodeFlag> for DagNodeFlags {
  fn from(value: DagNodeFlag) -> Self {
    DagNodeFlags(value as u32)
  }
}

impl std::ops::BitOr for DagNodeFlag {
  type Output = DagNodeFlags;

  fn bitor(self, rhs: Self) -> Self::Output {
    DagNodeFlags(self as u32 | rhs as u32)
  }
}

impl std::ops::BitOr<DagNodeFlags> for DagNodeFlag {
  type Output = DagNodeFlags;

  fn bitor(self, rhs: DagNodeFlags) -> Self::Output {
    DagNodeFlags(self as u32 | rhs.0)
  }
}


#[derive(Copy, Clone, PartialEq, Eq, Default, Hash, Debug)]
pub struct DagNodeFlags(pub(crate) u32);

impl DagNodeFlags {
  #[allow(non_upper_case_globals)]
  pub const RewritingFlags: DagNodeFlags = DagNodeFlags(1u32 | 4u32 | 8u32 | 16u32);

  // pub const RewritingFlags: DagNodeFlags = DagNodeFlag::Reduced | DagNodeFlag::Unrewritable |
  // DagNodeFlag::Unstackable | DagNodeFlag::Ground;

  pub fn set_copied_flags(&mut self, other_flags: DagNodeFlags) {
    self |=
      (DagNodeFlag::Reduced | DagNodeFlag::Unrewritable | DagNodeFlag::Unstackable | DagNodeFlag::Ground) & other_flags;
  }
}


impl DagNodeFlags {
  #[inline(always)]
  pub fn is_reduced(&self) -> bool {
    (self.0 & DagNodeFlag::Reduced as u32) != 0
  }

  #[inline(always)]
  pub fn is_copied(&self) -> bool {
    (self.0 & DagNodeFlag::Copied as u32) != 0
  }

  #[inline(always)]
  pub fn is_unrewritable(&self) -> bool {
    (self.0 & DagNodeFlag::Unrewritable as u32) != 0
  }

  #[inline(always)]
  pub fn is_unstackable(&self) -> bool {
    (self.0 & DagNodeFlag::Unstackable as u32) != 0
  }

  #[inline(always)]
  pub fn is_ground(&self) -> bool {
    (self.0 & DagNodeFlag::Ground as u32) != 0
  }

  #[inline(always)]
  pub fn is_hash_valid(&self) -> bool {
    (self.0 & DagNodeFlag::HashValid as u32) != 0
  }
}
