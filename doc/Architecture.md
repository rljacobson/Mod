Portions of this document are quoted from [Clavel et al.], which is published under the [Attribution-NonCommercial-NoDerivs 3.0 Unported (CC BY-NC-ND 3.0)](https://creativecommons.org/licenses/by-nc-nd/3.0/) license.

> Clavel, M., Eker, S., Lincoln, P., & Meseguer, J. (1996). Principles of Maude. Electronic Notes in Theoretical Computer Science, 4, 65-89. doi: 10.1016/S1571-0661(04)00034-9

# Architecture

```mermaid
classDiagram
direction RL
namespace Base System {
    class Core {
        Sort
        VariableSymbol
        Variable
        Equation
        Substitution
        ...
    }
    class Theory {
      Symbol
      DagNode
      Term
      LHSAutomaton
    }
}

Core <|-- Theory
Theory <|.. Core

namespace Rewrite Theories {
    class FreeTheory {
        FreeSymbol
        FreeDagnode
        FreeTerm
    }
    class CTheory{
        CSymbol
        CDagnode
        CTerm
    }
    class AUTheory{
        AUSymbol
        AUDagnode
        AUTerm
    }
    class AUCTheory{
        AUCSymbol
        AUCDagnode
        AUCTerm
    }
}


FreeTheory <|.. Core
CTheory <|.. Core
AUTheory <|.. Core
AUCTheory <|.. Core


FreeTheory <|-- Theory
CTheory <|-- Theory
AUTheory <|-- Theory
AUCTheory <|-- Theory

namespace Modules {
    class BOOL {
        EqualitySymbol
        BranchSymbol
        SortTestSymbol
    }
    class META-THEORY {
        MetaRewriteTheory
    }
}

BOOL <|-- FreeTheory
META-THEORY <|-- FreeTheory

class FrontEnd {
    Lexer
    Parser
    REPL
}

class GarbageCollector
```

For most uses terms are represented as trees, in which nodes are decorated with all kinds of information to simplify
parse time analysis For the subject term being rewritten, however, a directed acyclic graph DAG representation is used
with very compact nodes.

The innermost layer consists of the modules _Core_ ("Core Facilities") and Theory ("Theory API"). The Theory Interface
consists of abstract classes for basic objects whose concrete realization will differ for different equational theories,
such as: symbols, dag nodes, terms, lefthand side automata (for matching), righthand side automata (for constructing and
normalizing righthand side and condition instances), matching subproblems and matching extension information. Some of
the classes in the Theory Interface contain some concrete data and function members to provide useful common
functionality to derived classes. The Core Facilities module consists of concrete classes for basic objects that are
independent of the different equational theories, such as: sorts, connected components (kinds), variable symbols,
variables (as terms), equations, sort constraints, rules, sequences of matching subproblems and substitutions. Neither
the Core Facilities nor the Theory Interface treat any sort, symbol or equational theory as special in any way
whatsoever; all are manipulated through virtual functions in the abstract classes belonging to the Theory Interface. In
particular, this means that the code that handles conditional equations knows nothing about the Maude built in sort Bool
and its built in constants true and false. ...

Performance enhancing techniques implemented in the current prototype include:

 * Fixed size dag nodes for in-place replacement.
 * ull indexing for the topmost free function symbol layer of patterns; when the patterns for some free symbol only
   contain free symbols this is equivalent to matching a subject against all the patterns simultaneously.
 * Use of _greedy matching algorithms_, which attempt to generate a single matching substitution as fast as possible for
   patterns and subpatterns that are simple enough and whose variables satisfy certain conditions (such as not appearing
   in a condition). If a greedy matching algorithm fails it may be able to report that no match exists; but it is also
   allowed to report 'undecided' in which case the full matching algorithm must be used.
 * Use of binary search during AC matching for fast elimination of ground terms and previously bound variables.
 * se of a specially designed sorting algorithm which uses additional information to speed up the renormalization of AC
   terms.
 * Use of a Boyer-Moore style algorithm for matching under associative function symbols.
 * Compile time analysis of sort information to avoid needless searching during associative and AC matching.
 * ompile time analysis of non-linear variables in patterns in order to propagate constraints on those variables in an
   'optimal' way and reduce the search space.
 * Compile time allocation of fixed size data structures needed at run time.
 * Caching dynamically sized data structures created at run time for later reuse if they are big enough.
 * Bit vector encoding of sort information for fast sort comparisons.
 * Compilation of sort information into _regularity tables_ for fast incremental computation of sorts at run time.
 * fficient handling of _matching with extension_ through a theory independent mechanism that avoids the need for
   extension variables or equations.
