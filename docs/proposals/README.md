# HIR JSX props resolution prototype

Inactive review material requested by the rust-js team through agent-hub. The app does not apply this patch.

Base: rust-js 3839d978199e3578a7c3637e6daeb0b843432e22. Apply hir-jsx-props-resolution.patch to that revision for review; it is an alternative prototype, not an overlay for the newer upstream Scope implementation.

The syntax pass records unresolved wrapper signatures and provisionally emits a literal. After expansion, HIR resolves the input type to its struct. Only a macro in that struct’s defining module carrying the compiler-generated jsx_props marker is a builder. A changed decision triggers one replay before body checking; unchanged decisions stay in the original compiler session. Owned source identities and decisions cross sessions, not rustc IDs. Metadata, Clippy and lowering share the boundary.

The three regression cases pass on the base plus this prototype, including explicit imports, renames, qualified paths, a glob import, re-exports, literal Range/type-alias props, and unrelated same-name/lexical macros. Native metadata compilation is checked too. The broader earlier prototype run passed 219 focused JSX/Clippy/inventory tests. A 761-second complete gate failed on one redundant mutation and two inventories; the redundant rule was removed, the inventory corrected, and focused checks plus four new/retargeted mutations passed. The complete gate was not rerun to green. Details are retained in the patch’s dev-loop note.

Known limitation: discovery and replay can print early source warnings twice. The upstream team proposed combining resolved HIR authority with a cheap initial Scope guess and buffering discovery diagnostics. This prototype does not yet implement that combined design.
