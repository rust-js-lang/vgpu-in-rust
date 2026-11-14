# Compiler gap history

Current published compiler pin: `27e3f76eceecd50d3b2d839ecc8b22f3842a926c`.

All six previous downstream fixes are upstream-owned. No local compiler patch
is applied. The submodule checkout is clean; setup and compile use the published
compiler directly. Historical patches remain in this repository's Git history.

| Issue | Published implementation |
| --- | --- |
| #1481 Cow AsRef/Borrow | `91c41cd9a` |
| #1752 opaque LazyCell/LazyLock declarations | `6aa792201` |
| #1763 React hook handle declarations | `07d97ef66` |
| #1775 untagged conversion bodies | `a02f938b0` |
| #1778 nested derived Debug | `75abfed4b` |
| #1779 WebGPU flag namespaces | `7d1c48f3c` |

The earlier native Error-family, nullable-promise and async-declaration fixes
also remain upstream-owned. Future compiler gaps still follow AGENTS.md:
reproduce, file an issue, patch the private submodule, retain the patch here,
and adopt the upstream replacement when available.
