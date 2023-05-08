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

There are variations of the matching problem that are not implemented in Maude. For example, there are 
algorithms in the literature for many-to-one and many-to-many matching that, as far as I know, have no counterpart 
in Maude. These matching problems have important applications. Implementing these algorithms would be an excellent 
stretch goal for this project.  

## Building

It doesn't.

## License and Authorship

Copyright (c) 2022-2023 Robert Jacobson. This software library is distributed under the terms of the GNU Lesser General
Public License.

Portions of this software are derived from the source code of Maude 3.2.1. [Maude](https://github.com/SRI-CSL/Maude)
is copyright (c) 1997-2003 SRI International and is distributed under the terms of the GNU General Public License
version 2 or later.
