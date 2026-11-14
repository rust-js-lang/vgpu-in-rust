# Compiler fixes adopted upstream

The published submodule gitlink is the authority for the compiler revision.
All application patches are retired: the Battery API fix is published at
618662df7 (#1790), and imported JSX props are fixed at 77a03e8e5 (#1794).
The app now uses published 7977d6f205090e273de0601e0425fa233ae38e9f with no
downstream compiler source changes or patch-application helper.

The team requested the alternative HIR resolver prototype for follow-up glob
coverage. It is retained as an [inactive review proposal](../../docs/proposals/README.md),
not applied to the app or its compiler.

| Issue | Published implementation |
| --- | --- |
| #1481 Cow AsRef/Borrow | 91c41cd9a |
| #1752 opaque LazyCell/LazyLock declarations | 6aa792201 |
| #1763 React hook handle declarations | 07d97ef66 |
| #1775 untagged conversion bodies | a02f938b0 |
| #1778 nested derived Debug | 75abfed4b |
| #1779 WebGPU flag namespaces | 7d1c48f3c |
| #1784 generic JS erasure field liveness | 670a6bca3 |
| #1785 global binding function-pointer feature checks | c4aa4e5d5 |
| #1786 guarded classification of known binding casts | 4c58cdcf6 |
| #1790 Battery Status API and ambient declarations | 618662df7 |
| #1794 imported JSX props and literal foreign props | 77a03e8e5 |

Claude relayed #1786 through agent-hub. Its published implementation replaces
the last patch, downstream mutation and patch-application helper. Unknown
function pointers supplied by JS retain their value checks; known namespaced
bindings keep missing-root guards and known ordinary Rust functions stay refused.
The browser material tests still exercise actual createImageBitmap feature
checks, including a missing global. See docs/validation.md for evidence and the
remaining landing-page/deployment gates.
