# Memory Management

## Notes

Implementation Strategy: Anything that has a defined owner can be held in an `RcCell<T>`. Every other reference to it should be a 
`WeakCell<T>`. Thus, most `Rc*`s should be `Wk*`s.

## Code Map of Ownership

This data is far from complete. The two tables provide two different views. They describe the Maude C++ source code, not the Mod Rust source code.

| Object              | Owned By         | Owns          | Deleted By                                       |
|:--------------------|:-----------------|:--------------|--------------------------------------------------|
| `Term`              | parent `Term`    | child `Term`s |                                                  |
| `LHSAutomata`       | `NonGroundAlien` |               | `FreeLhsAutomaton` owns `nonGroundAliens` vector |
| `LHSAutomata`       | `NonGroundAlien` |               | `FreeRemainder` owns `nonGroundAliens` vector    |  
| `DagNode`           | `DagNodeMembers` | child nodes   | `DagNodeMembers` owns `args`                     |
| `Instruction`       | `CachedDag`      |               | `CachedDag` owns `instructionSequence`           |
| `Term`              | `BranchSymbol`   | child nodes   | `BranchSymbol` owns `testSymbols`                |
| `Term`              | `Equation`       | child nodes   | `Equation` owns `rhs`                            |
| `Instruction`       | `Equation`       |               | `Equation` owns `instructionSequence`            |
| `LHSAutomaton`      | `PreEquation`    |               | `PreEquation` owns `lhsAutomaton`                |
| `Term`              | `PreEquation`    |               | `PreEquation` owns `lhs`                         |
| `ConditionFragment` | `PreEquation`    |               | `PreEquation` owns `condition` vector            |
| `LHSAutomaton`      | `PreEquation`    |               | `PreEquation` owns `lhs`                         |





| Destructor      | Deleted name          | Deleted type        | by                   |
| :-------------- | :-------------------- | ------------------- | -------------------- |
| `~Rule`         | `nonExtLhsAutomaton`  | `LHSAutomaton`      | d                    |
| '               | `extLhsAutomaton`     | `LHSAutomaton`      | d                    |
| '               | `rhs`                 | `Term`              | `deepSelfDestruct()` |
| `~BranchSymbol` | `testTerms[i]`        | `Term`              | `deepSelfDestruct()` |
| `~CachedDag`    | `instructionSequence` | `Instruction`       | d                    |
| '               | `term`                | `Term`              | `deepSelfDestruct()` |
| `~Equation`     | `rhs`                 | `Term`              | `deepSelfDestruct()` |
| '               | `instructionSequence` | `Instruction`       | d                    |
| `~PreEquation`  | `lhsAutomaton`        | `LHSAutomaton`      | d                    |
| '               | `lhs`                 | `Term`              | `deepSelfDestruct()` |
| '               | `condition[i]`        | `ConditionFragment` | d                    |
|                 |                       |                     |                      |
