# Compiler fixes adopted upstream

The published submodule gitlink is the authority for the compiler revision.
The Battery Status API fix is published at 618662df7 (#1790), so its patch and application helper are retired. No downstream compiler source patch remains. The imported JSX props wrapper regression for [#1794](https://github.com/rust-js-lang/rust-js/issues/1794) is retained under regressions; its fix is pending.

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

Claude relayed #1786 through agent-hub. Its published implementation replaces
the last patch, downstream mutation and patch-application helper. Unknown
function pointers supplied by JS retain their value checks; known namespaced
bindings keep missing-root guards and known ordinary Rust functions stay refused.
The browser material tests still exercise actual createImageBitmap feature
checks, including a missing global. See docs/validation.md for evidence and the
remaining landing-page/deployment gates.
