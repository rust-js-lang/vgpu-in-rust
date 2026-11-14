# Downstream compiler fix

The gitlink is the authority for the published compiler revision:
`59b8850fffaee6c9eb33b56753f5646d50d4df47`.

The single retained patch fixes [#1785](https://github.com/rust-js-lang/rust-js/issues/1785):
a global binding cast to a function pointer retains its known function origin
when asked with is_defined/is_function. The same origin owner serves standalone
and tuple-path casts. Unknown/reassigned pointers are not guessed. Missing
globals are neither evaluated nor called before their typeof check.

Run `bun run patch:compiler` after submodule initialization. The script applies
only to the pinned revision and proves an already-applied stack in a disposable
copy. The regression proves absent/present/nonfunction globals, namespaced
guards, one getter read, zero calls and nonbinding/ambiguous refusals. Five new
or retargeted mutations are caught. Documentation is included in the patch.

Earlier fixes are upstream-owned and no longer overridden:

| Issue | Published implementation |
| --- | --- |
| #1481 Cow AsRef/Borrow | `91c41cd9a` |
| #1752 opaque LazyCell/LazyLock declarations | `6aa792201` |
| #1763 React hook handle declarations | `07d97ef66` |
| #1775 untagged conversion bodies | `a02f938b0` |
| #1778 nested derived Debug | `75abfed4b` |
| #1779 WebGPU flag namespaces | `7d1c48f3c` |
| #1784 generic JS erasure field liveness | `670a6bca3` |

The upstream liveness fix makes strict application Clippy pass with no warnings
or suppressions. Historical patches remain in Git history. Validation and
outstanding milestone gates are recorded in docs/validation.md.
