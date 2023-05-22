/*!

Discrimination net for the Free theory.

Used for rewriting.

*/

use std::{collections::HashSet, default};

use crate::{
    abstractions::WeakCell,
    core::module::ModuleItem,
    theory::{
        DagNode,
        RcDagNode,
        RcSymbol,
        NodeList
    },
};

use super::{FreeRemainder, RcFreeRemainder, FreeRemainderList};

pub type PatternSet = HashSet<i32>;
pub type RcFreeNet  = WeakCell<FreeNet>;


struct Triple {
    symbol : RcSymbol,
    slot   : i32,
    subtree: i32,
}

impl Eq for Triple {}

impl PartialEq for Triple {
    fn eq(&self, other: &Self) -> bool {
        self.symbol.get_index_within_module() == other.symbol.get_index_within_module()
    }
}

impl PartialOrd for Triple {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.symbol
                .get_index_within_module()
                .cmp(&other.symbol.get_index_within_module()),
        )
    }
}

impl Ord for Triple {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.symbol
            .get_index_within_module()
            .cmp(&other.symbol.get_index_within_module())
    }
}

#[derive(Copy, Clone)]
struct TestNode {
    /// Index of next test node to take for > and < cases (-ve encodes index of applicable list, 0 encodes failure)
    notEqual: (i32, i32),
    /// Itack slot to get free dagnode argument list from (-1 indicates use old argument)
    position: i32,
    /// Index of argument to test
    argIndex: i32,
    /// Index within module of symbol we test against
    symbolIndex: i32,
    /// Index of stack slot to store free dagnode argument list in (-1 indicates do not store)
    slot: i32,
    /// Index of next test node to take for == case (-ve encode index of applicable list)
    equal: i32,
}

#[derive(Default)]
pub struct FreeNet {
    // TODO: Stack should be `Vec<DagNode**>`, a vector of a list of dag node pointers?
    stack: Vec<NodeList>,
    net: Vec<TestNode>,
    fastApplicable: Vec<FreeRemainderList>,
    remainders: FreeRemainderList,
    applicable: Vec<PatternSet>,
    fast: bool,
}

impl FreeNet {
    pub fn new() -> Self {
        FreeNet {
            fast: true,
            ..FreeNet::default()
        }
    }
}
