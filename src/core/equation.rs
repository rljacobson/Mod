use crate::theory::{
    RcLHSAutomaton,
    RcTerm
};

pub struct Equation {
    label: u32,
    lhs: RcTerm,
    lhsAutomaton: RcLHSAutomaton,
    // lhsDag: DagRoot,  // For unification
    // condition: Vec<ConditionFragment>,
}
