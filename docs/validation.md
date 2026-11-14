# Validation evidence

The complete landing page is not deployed. These checks prove scene mathematics,
React leaves and GPU building blocks; they do not prove a completed landing page.

## Current checkpoint

The published rust-js gitlink is
`ecb588210201077d00db0935274d49009a52e2ce` (2026-11-14). A single reproducible
patch retains six outstanding fixes. No private compiler commit is the gitlink.
The upstream nullable-promise, async-declaration, basic derived-Debug and native
Error-family fixes are used unchanged.

Passing application checks:

- 21 native Rust tests and the same 21 compiled JavaScript tests.
- 36 app tests with more than 157,000 assertions: scene optics, framing,
  interaction, exact rounded mesh vertices/topology, exact Float32 camera
  matrices, spectral layouts/vertices/flux, retained upload identity, native
  RangeErrors, React markup/context, retained assets and complete compiler-patch
  reproducibility.
- Runtime controls match the complete upstream default schema. Lamp interpolation,
  portrait wall overscan and scene/glass uniform values match original functions
  across themes, camera settings, mesh tiers, colors, reveal clamps and debug
  gating. Projection, environment and plane arrays retain shared identities.
- Reflected packing matches upstream bytes for the actual Scene/Glass WGSL
  structs, scalar/vector/matrix padding, nested arrays and binary16 rounding.
  Invalid input preserves diagnostic payloads; changing getters preserve
  validation/write order, coercion and native exceptions. Rust callers receive
  structured Result errors; a complete JavaScript SDK error-class port is not
  claimed.
- Invalid uniform values allocate no GPU resource. Rejected updates preserve the
  previous upload, and valid updates retain the same buffer.
- 18 production-built Chromium checks. Ten cover setup-tab parity, copy/reset,
  rapid transitions, layout stability, unavailable/denied clipboard, reduced
  motion, mobile sizing and unmount cleanup. Eight require a real WebGPU adapter:
  all 44 retained shaders compile; malformed shader diagnostics work; the Rust
  fullscreen pass copies exact pixels/orientation; the additive light pass
  renders red/blue spectral output and disappears with zero reveal; both glass
  interface pipelines compile for single-sample and 4x MSAA targets. Connected composition is now also checked, as detailed below.
- Native core Clippy and the rust-js-aware web Clippy pass. Existing generated
  modules remain unchanged apart from newly added exports and functionality.

WGSL/assets remain byte-identical to upstream. Rust owns module diagnostics,
pipelines, command encoding, uniform values, reflected packing and uniform/vertex
buffer allocation/upload. The existing WGSL asset compiler owns shader imports
and reflection at build time. Upstream app TypeScript is used only by isolated
reference tests.

## Retained compiler fixes

- #1481: Cow AsRef/Borrow dictionaries use the Cow dereference owner. Native
  corpus comparisons cover strings, slices, arrays, paths and both variants.
- #1752: LazyCell/LazyLock declarations use the Pending/Ready storage schema.
  Strict declaration and runtime-state checks pass.
- #1763: React hook handles preserve payload types and React 18/19 ref aliases.
  An upstream replacement is available; it will be adopted at a grouped upgrade.
- #1775: arbitrary untagged From/Into bodies preserve their Rust behavior and
  side effects. Library bodies remain exported with private state owned there;
  pure wrappers are proved rather than assumed. Nine existing React/Next
  boundary wrappers explicitly use the established identity contract. Standard
  trait resolution lives in recognition; the untagged owner classifies bodies.
- #1778: nested derived Debug field coercions stay inside their builder, rather
  than being treated as values passed to unrelated foreign writers. Actual
  framed-writer propagation retains its existing path.
- #1779: WebIDL supplies the standard GPU flag namespaces, numeric types and
  native property reads. The import regression failed before the fix and passes
  afterward. Three isolated generator probes remove namespace selection, member
  retention and constant emission independently; each loses the required module.
  These probes are retained in test/webapi-gpu.test.ts, not falsely represented
  as runs of the compiler mutation harness.

Eight conversion/Debug/dispatch/reachability mutations are caught on the corrected
stack. This includes five new or retargeted semantic probes and three existing
sites updated after the refactor. The direct-From live-borrow test catches the
pure-effects mutation; its earlier Into form survived and is not counted as
proof of that analysis path. Four native/library conversion probes still pass.
Upstream's standard Error class-identity test also passes unchanged; the old
RangeError-only override has been removed.

## Broad compiler gate

The ecb5882 `check:fast --all` run completed in 709 seconds with seven failed test
files: Next integration, three JSX files, architecture, mutation inventory and
CI mutation-site validation. All fourteen corpus parts, added mutations,
compiler Clippy, native unit tests, formatting, the Vite example, crate corpus
and reported matrix/sample stages completed. This was a failed gate, not a
passing suite or a timeout.

After correcting binding contracts, ownership and stale mutation sites, all
seven failed files pass in focused reruns: 212 tests, including Next build,
compile, dev refresh and error recovery. The four native/library conversion
probes and all eight affected mutations also pass. A complete passing broad
rerun remains unproven; focused reruns are not represented as that result.

Earlier pins had interrupted or timed-out broad runs, and one 546-second run
reported an architecture failure later corrected. None is counted as passing.
The old child-startup race was reported as #1770 and is now fixed upstream.

## Remaining milestone gates

Finish the remaining prism scene graph, including bloom,
light/dark composition, quality policy and resource lifecycle. Finish the full
landing root and every section. Compare full-page GPU output, themes,
desktop/mobile framing, interaction, fallback and cleanup against upstream.
Obtain the remaining broad compiler evidence. Publish the source, enable
workflow-based Pages, install a build/deploy-only workflow with current actions
pinned to full commits, and verify the actual public page with visual evidence.

## Connected environment and glass checkpoint

Rust now owns offscreen/MSAA targets, reflected resource slots, retained uniform
buffers, rounded indexed GPU geometry, and authored studio/debug environment
baking with successive horizontal/vertical mip filtering. Group-zero shader
inputs are validated before any uniform buffer allocation; rebinding preserves
those buffers while texture views change. Draw helpers record commands into the
graph-owned pass, so exterior light, back glass, internal light and front glass
retain the original composition order. Fullscreen effects support the target
sample count, including the scene's 4x MSAA copy stage.

Real GPU comparisons show zero differing half-float channels across all eight
mip levels for both environments and across a connected spectral/glass HDR
scene. The comparison uses the original SDK, original environment/geometry code
and independently generated upstream spectral mesh on the same device. A
test-only gateway exports actual upstream functions; no mock renderer or
production TypeScript implementation is substituted. Nonzero/finite checks
prevent equal blank images from passing.

Repeated preparation allocates/uploads/submits nothing, unchanged target sizes
retain all resource identities, resizing destroys the previous resolved/MSAA
textures, and readback verifies the new MSAA resolve pixels. Unit checks cover
preflight failure without allocations, duplicate/missing bindings, nonsequential
reflection slots, retained uploads and cleanup. The checkpoint's scoped tests
pass: 36 app tests (over 157,000 assertions), 18 production browser tests, and
rust-js-aware web Clippy. No compiler source changes were required for these
features; f32/u16 vector uploads already work through the existing Takes rule.

This remains a renderer checkpoint. Bloom, final presentation/particles, light
mode, full lifecycle/interaction integration, the complete landing page and
public Pages deployment are still required. The older broad compiler gate
status above is unchanged.
