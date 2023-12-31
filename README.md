# Mod: An experimental Maude implementation

> **[Maude](https://github.com/SRI-CSL/Maude)** is a high-performance reflective language and system supporting both equational and rewriting logic
> specification and programming for a wide range of applications.

Maude is interesting in part because it implements some of the most performant and sophisticated pattern matching
algorithms that are known. Some of the algorithms are described across the literature. (See the 
[Bibliography](doc/Bibliography.md).) The most important references are:

* S. Eker, _Fast matching in combinations of regular equational theories_, Electronic Notes in Theoretical Computer
  Science, 1996,
  vol. 4, p. 90-109, ISSN 1571-0661, https://doi.org/10.1016/S1571-0661(04)00035-0.
* S. Eker,
  _Associative-commutative matching via bipartite graph matching_,
  Computer Journal, 38 (5) (1995), pp. 381-399

The algorithms are complicated. Maude is implemented in C++. The code is excellent. However, because Maude was
designed to be modular, and because of the algorithms in \[Eker 1996] that allow combinations of theories, the
algorithms for matching are somewhat obscured. In other words, you can't just copy and paste the algorithm into your
own code.

Thus, I am attempting to reimplement the algorithms in Rust and hopefully clarify some of the implementation details
at the same time.

## Project Status

The Free Theory is the first to be implemented. Most of the core algorithms and infrastructure are in place. 
The implementation of memoization and hash consing has been started, but I've decided I want to see the basic core 
functionality work before I implement and/or enable these essential optimizations. 

Thus, for now, **most development is going to happen on the [`core` branch](https://github.com/rljacobson/Mod/tree/core)** in which optimizations have been disabled. 
The `main` branch will preserve the work started on hash consing and memoization. Evaluation strategies might also be 
left out of `core`.

## Building

It does build sometimes. It doesn't work yet, but it's super close.

## Vision and Future Work

The primary goal of this project is to better understand the matching algorithms in Maude. There is always the fantasy,
though, that this library will one day be an industrial strength, state-of-the-art expression matching library suitable
for use in term rewriting systems, computer algebra systems, SMT-solvers, and so forth. In other words, it could be to
expression matching libraries what Maude itself is to term rewriting systems. It has taken world experts many decades to
get Maude to its current status. Even extracting Maude's algorithms to a Rust codebase and exposing a generic API for
them is a nontrivial task.

There are variations of the matching problem that are not implemented in Maude. For example, there are
algorithms in the literature for many-to-one and many-to-many matching that, as far as I know, have no counterpart
in Maude. These matching problems have important applications. Implementing these algorithms would be an excellent
stretch goal for this project.

## License and Authorship

Copyright (c) 2022-2023 Robert Jacobson. This software library is distributed under the terms of the GNU Lesser General
Public License.

The algorithms implemented in this software are primarily due to Steven Eker (see bibliography), who is also the
principle developer of Maude.

Portions of this software are derived from the source code of Maude 3.2.1. [Maude](https://github.com/SRI-CSL/Maude), 
which is copyright (c) 1997-2003 SRI International and is distributed under the terms of the GNU General Public License
version 2 or later.
