# Downstream compiler fixes

The gitlink is the authority for the compiler revision. Current published pin:
`0fb07aed694681295a1f2132ee78fbf647f764ed` (fetched 2026-11-14).

The single `0001-landing-boundary-fixes.patch` retains four outstanding fixes:

| Upstream issue | Change |
| --- | --- |
| [#1773](https://github.com/rust-js-lang/rust-js/issues/1773) | Native RangeError constructor preserves prototype identity, optional message/cause, and thrown values. |
| [#1481](https://github.com/rust-js-lang/rust-js/issues/1481) | Cow AsRef/Borrow dictionaries consume the existing Cow dereference owner. |
| [#1752](https://github.com/rust-js-lang/rust-js/issues/1752) | Lazy storage declarations use the representation owner's Pending/Ready schema. |
| [#1763](https://github.com/rust-js-lang/rust-js/issues/1763) | React hook handles retain their types, with the correct React 18/19 mutable-ref alias. |

Run `bun run patch:compiler` after initializing the submodule. An already-applied
stack is proven by reversing a disposable copy; the actual checkout is never
reversed. A conflicting edit or wrong revision is rejected. Keep the gitlink at
a published commit; the checkout remains intentionally dirty after patching.

The upgrade removed the old Cow declaration, metamorphic partition, Cargo-input,
memory-fixture and validation-pool patches. Upstream now owns those fixes:
`6609623`, `4ea9947`, `4db5a47`, `c53afcd`, `bb0bdee`, and `ddead01`.
The last also adds shared machine leases and stale-owner-safe locks.
Upstream `b0bc0a3` now owns the nullable WebAPI promise fix (#1768), its
generator changes and regression. Upstream `0fb07ae` also owns async/opaque
Future declaration payloads (#1769). Neither fix has a downstream override. Old patches
remain in Git history; they are not applied on top of the new implementations.

See `docs/validation.md` for scoped results and the upgrade's validation status.
Interrupted broad checks are never counted as passing.
