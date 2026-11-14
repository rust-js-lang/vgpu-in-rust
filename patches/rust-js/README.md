# Downstream compiler fixes

The gitlink is the authority for the compiler revision. Current pin: `0f641c21350f3719f383dd636385c65c7a82eaec`.

| Patch | Upstream issue | Change |
| --- | --- | --- |
| 0001-cow-declarations.patch | [#1748](https://github.com/rust-js-lang/rust-js/issues/1748) | Declare Cow's actual borrowed/owned payload union using existing enum and recognition owners. |
| 0002-parallel-metamorphic-check.patch | [#1749](https://github.com/rust-js-lang/rust-js/issues/1749) | Partition the full default metamorphic sample in the existing validation pool, without dropping cases. |
| 0003-cow-as-ref-borrow.patch | [#1481](https://github.com/rust-js-lang/rust-js/issues/1481) | Supply Cow's AsRef/Borrow dictionaries through the existing Cow dereference owner. |
| 0004-cargo-settings-input.patch | [#1751](https://github.com/rust-js-lang/rust-js/issues/1751) | Track the settings manifest as a common build input, so Cargo metadata-only runner edits invalidate old artifacts. |
| 0005-bounded-memory-fixture.patch | [#1754](https://github.com/rust-js-lang/rust-js/issues/1754) | Retain memory pressure while removing a redundant linear-time count from its fixture. |
| 0006-typed-lazy-storage.patch | [#1752](https://github.com/rust-js-lang/rust-js/issues/1752) | Declare lazy initializer/payload states from the existing representation owner's schema. |
| 0007-backfill-check-pool.patch | [#1762](https://github.com/rust-js-lang/rust-js/issues/1762) | Track active reservations and backfill runnable jobs within the existing budgets. |
| 0008-react-handle-types.patch | [#1763](https://github.com/rust-js-lang/rust-js/issues/1763) | Preserve typed React hook handles and the correct React 18/19 mutable-ref aliases. |

Run `bun run patch:compiler` after initializing the submodule. A complete already-applied stack is proven by reversing a disposable copy; the actual checkout is never reversed. A different compiler revision or conflicting edits are rejected. The submodule remains intentionally dirty after applying tracked patches. Do not commit an unpublished compiler revision as its gitlink.

Regressions failed before their fixes. The declaration and dictionary changes include meaningful mutations. The runner-input mutation covers both Cargo and standalone compiles. The stack includes regression inventory notes in TESTS.md.

The first broad compiler check timed out after 720 seconds with the metamorphic sample still active. That attempt is not a passing result or a controlled performance benchmark. See `docs/validation.md` for current evidence.
