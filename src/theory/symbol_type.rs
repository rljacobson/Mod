/*!

A `SymbolType` is a bitfield of symbol attributes.

*/

const FLAG_MASK : u32 = 0xffffff;
const TYPE_SHIFT: u32 = 24;

// ToDo: It's a little weird that `BasicSymbolTypes` aren't `SymbolTypes`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BasicSymbolTypes {
    Standard,

    //	System created symbols.
    Variable,
    SortTest,
    InternalTuple,

    //	Special properties.
    SystemTrue,
    SystemFalse,
    Bubble,

    //	Special symbols that do not deal with attachments.
    Float,
    String,

    //	Special symbols that do deal with attachments.
    BranchSymbol,
    EqualitySymbol,
    FloatOp,
    StringOp,
    QuotedIdentifier,
    QuotedIdentifierOp,
    ModelCheckerSymbol,
    SatSolverSymbol,
    MetaLevelOpSymbol,
    LoopSymbol,
    SuccSymbol,
    MinusSymbol,
    NumberOpSymbol,
    AcuNumberOpSymbol,
    CuiNumberOpSymbol,
    DivisionSymbol,
    RandomOpSymbol,
    MatrixOpSymbol,
    CounterSymbol,
    SocketManagerSymbol,
    InterpreterManagerSymbol,
    SmtSymbol,
    SmtNumberSymbol,
    FileManagerSymbol,
    StreamManagerSymbol,
    DirectoryManagerSymbol,
    ProcessManagerSymbol,
    TimeManagerSymbol,
    ObjectConstructorSymbol,

    EndOfSymbolsWithAttachments,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u32)]
pub enum SymbolAttribute {
    // Syntactic attributes.
    Prec = 0x1,
    Gather = 0x2,
    Format = 0x4,
    Latex = 0x8,

    // Semantic attributes.
    Strat = 0x10,
    Memo = 0x20,
    Frozen = 0x40,
    Ctor = 0x80,

    // OO attributes.
    Config = 0x100,
    Object = 0x200,
    Message = 0x400,
    MsgStatement = 0x800, // MESSAGE flag was set by msg statement rather than an attribute; only used by SyntacticPreModule

    // Theory attributes.
    Assoc = 0x1000,
    Comm = 0x2000,
    LeftId = 0x4000,
    RightId = 0x8000,
    Idem = 0x10000,
    Iter = 0x20000,

    //	Misc.
    PConst = 0x200000,
    Poly = 0x400000,
    Ditto = 0x800000,
}

impl SymbolAttribute {
    #![allow(non_upper_case_globals)]
    const Axioms: SymbolType = SymbolType(
        SymbolAttribute::Assoc as u32
            | SymbolAttribute::Comm as u32
            | SymbolAttribute::LeftId as u32
            | SymbolAttribute::RightId as u32
            | SymbolAttribute::Idem as u32,
    );

    const Collapse: SymbolType = SymbolType(
        SymbolAttribute::LeftId as u32
            | SymbolAttribute::RightId as u32
            | SymbolAttribute::Idem as u32,
    );

    /// Simple attributes are just a flag without additional data. They produce a warning if given twice.
    const SimpleAttributes: SymbolType = SymbolType(
        SymbolAttribute::Assoc as u32
            | SymbolAttribute::Comm as u32
            | SymbolAttribute::Config as u32
            | SymbolAttribute::Ctor as u32
            | SymbolAttribute::Idem as u32
            | SymbolAttribute::Iter as u32
            | SymbolAttribute::Memo as u32
            | SymbolAttribute::Message as u32
            | SymbolAttribute::Object as u32
            | SymbolAttribute::PConst as u32
    );

    /// All flagged attributes except ctor, poly, ditto. They need to agree between declarations of an operator.
    const Attributes: SymbolType = SymbolType(
              SymbolAttribute::Axioms.0
            | SymbolAttribute::Config as u32
            | SymbolAttribute::Format as u32
            | SymbolAttribute::Frozen as u32
            | SymbolAttribute::Gather as u32
            | SymbolAttribute::Iter as u32
            | SymbolAttribute::Latex as u32
            | SymbolAttribute::Memo as u32
            | SymbolAttribute::Message as u32
            | SymbolAttribute::Object as u32
            | SymbolAttribute::PConst as u32
            | SymbolAttribute::Prec as u32
            | SymbolAttribute::Strat as u32
    );
}

pub struct SymbolType(u32);

impl SymbolType {
    pub fn is_set(&self, flag: SymbolAttribute) -> bool {
        return (self.0 & flag as u32) != 0;
    }

    pub fn all_set(&self, flags: SymbolType) -> bool {
        return (self.0 & flags.0) == flags.0;
    }

    pub fn any_set(&self, flags: SymbolType) -> bool {
        return (self.0 & flags.0) != 0;
    }

    pub fn set(&mut self, flags: SymbolType) {
        self.0 = self.0 | flags.0;
    }
}

impl std::ops::BitOr<SymbolAttribute> for SymbolType {
    type Output = Self;

    fn bitor(self, rhs: SymbolAttribute) -> Self::Output {
        SymbolType(self.0 | rhs as u32)
    }
}

impl std::ops::BitOr for SymbolType {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        SymbolType(self.0 | rhs.0)
    }
}

impl From<SymbolAttribute> for SymbolType {
    fn from(value: SymbolAttribute) -> Self {
        SymbolType(value as u32)
    }
}

impl std::ops::BitOr for SymbolAttribute {
    type Output = SymbolType;

    fn bitor(self, rhs: Self) -> Self::Output {
        SymbolType(self as u32 | rhs as u32)
    }
}
