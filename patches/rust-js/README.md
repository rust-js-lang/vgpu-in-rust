# Compiler fixes adopted upstream

The published submodule gitlink is the authority for the compiler revision.
The current pin is c4aa4e5d54028aa834b0892838e22d3cb7bf5649.
One minimal patch remains for [#1786](https://github.com/rust-js-lang/rust-js/issues/1786):
function-value classification consumes the existing cast-origin owner before
choosing a caller-value check. This preserves missing-root guards for known
namespaced bindings and refuses known ordinary Rust function casts consistently.
Run bun run patch:compiler after submodule initialization. Patch application is
proved against the pin in a disposable checkout. Earlier fixes remain in history.

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

Claude relayed #1785 through agent-hub. Its published implementation replaces
the older #1785 lowering and mutation overrides. A broader regression uncovered
the separate #1786 classification gap; its test and fix are retained in the new patch.
The real browser material tests exercise the actual createImageBitmap function
pointer check, including an absent global. Unknown and ordinary Rust function
pointers are still refused by the compiler. See docs/validation.md for evidence
and the remaining landing-page/deployment gates.
