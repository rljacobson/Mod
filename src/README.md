# Mod: An experimental Maude implementation

> **[Maude](https://github.com/SRI-CSL/Maude)** is a high-performance reflective language and system supporting both equational and rewriting logic 
> specification and programming for a wide range of applications. 

Maude is interesting in part because it implements some of the most performant and sophisticated pattern matching 
algorithms that are known. Some of the algorithms are described across the literature:

* ker, S.: _Fast sort computations for order-sorted matching and unification_. In: Formal Modeling:
  Actors, Open Systems, Biological Systems - Essays Dedicated to Carolyn Talcott on the Occasion
  of Her 70th Birthday. vol. 7000, pp. 299–314. Springer LNCS (2011)
* Dur  ́an, F., Eker, S., Escobar, S., Mart ́ı-Oliet, N., Meseguer, J., Talcott, C.L.: _Associative unification and
  symbolic reasoning modulo associativity in maude._ In: Rewriting Logic and Its Applications - 12th
  International Workshop, WRLA 2018, Held as a Satellite Event of ETAPS, Thessaloniki, Greece, June
  14-15, 2018, Proceedings. Lecture Notes in Computer Science, vol. 11152, pp. 98–114. Springer (2018)
* S. Eker, _Fast matching in combinations of regular equational theories_, Electronic Notes in Theoretical Computer 
  Science, 1996,
  vol. 4, p. 90-109, ISSN 1571-0661, https://doi.org/10.1016/S1571-0661(04)00035-0.
* S. Eker,
  _Associative-commutative matching via bipartite graph matching_,
  Computer Journal, 38 (5) (1995), pp. 381-399
* S. Eker. _Associative matching for linear terms_. Technical Report CS-R9224, Center for Mathematics and Computer Science, Amsterdam, July 1992.
* Eker, S. _Single Elementary Associative-Commutative Matching_. Journal of Automated Reasoning 28, 35–51 (2002). https://doi.org/10.1023/A:1020122610698

The algorithms are complicated. Maude is implemented in C++. The code is excellent. However, because Maude was 
designed to be modular, and because of the algorithms in \[Eker 1996] that allow combinations of theories, the 
algorithms for matching are somewhat obscured. In other words, you can't just copy and paste the algorithm into your 
own code.

Thus, I am attempting to reimplement the algorithms in Rust and hopefully clarify some of the implementation details 
at the same time.

## Building

It doesn't.

## License and Authorship

Copyright (c) 2022 Robert Jacobson. This software library is distributed under the terms of the GNU Lesser General 
Public License. 

Portions of this software are derived from the source code of Maude 3.2.1. [Maude](https://github.com/SRI-CSL/Maude) 
is copyright (c) 1997-2003 SRI International and is distributed under the terms of the GNU General Public License 
version 2 or later.
