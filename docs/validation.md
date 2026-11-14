# Validation evidence

The complete landing page is not deployed. These checks prove scene mathematics,
React leaves and GPU building blocks; they do not prove a completed landing page.

## Current checkpoint

The published rust-js gitlink is
`27e3f76eceecd50d3b2d839ecc8b22f3842a926c` (2026-11-14). All retained compiler patches are now
replaced by published upstream implementations. No private compiler commit is the gitlink.
The upstream nullable-promise, async-declaration, basic derived-Debug and native
Error-family fixes are used unchanged.

Passing application checks:

- 26 native Rust tests and the same 26 compiled JavaScript tests.
- 40 app tests with 160,463 assertions: scene optics, framing,
  interaction, exact rounded mesh vertices/topology, exact Float32 camera
  matrices, spectral layouts/vertices/flux, retained upload identity, native
  RangeErrors, React markup/context, retained assets, bloom kernels/pairing and
  theme/intro reveal timing.
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
- 27 production-built Chromium checks. Ten cover setup-tab parity, copy/reset,
  rapid transitions, layout stability, unavailable/denied clipboard, reduced
  motion, mobile sizing and unmount cleanup. Seventeen require a real WebGPU adapter:
  all 44 retained shaders compile; malformed shader diagnostics work; the Rust
  fullscreen pass copies exact pixels/orientation; the additive light pass
  renders red/blue spectral output and disappears with zero reveal; both glass
  interface pipelines compile for single-sample and 4x MSAA targets. Connected composition is now also checked, as detailed below.
- Native core Clippy previously passed. The rust-js-aware Clippy command exits
  successfully, but the new generic parameter-erasure boundary exposes four
  false-positive dead-code warnings; the isolated failing regression is filed
  as #1784. No suppression or visibility workaround is applied. LazyLock type
  declarations now use the published opaque @rust-js/std type.

WGSL/assets remain byte-identical to upstream. Rust owns module diagnostics,
pipelines, command encoding, uniform values, reflected packing and uniform/vertex
buffer allocation/upload. The existing WGSL asset compiler owns shader imports
and reflection at build time. Upstream app TypeScript is used only by isolated
reference tests.

## Earlier compiler checkpoint (ecb5882)

- #1481: Cow AsRef/Borrow dictionaries use the Cow dereference owner. Native
  corpus comparisons cover strings, slices, arrays, paths and both variants.
- #1752: LazyCell/LazyLock declarations use the Pending/Ready storage schema.
  Strict declaration and runtime-state checks pass.
- #1763: React hook handles preserve payload types and React 18/19 ref aliases.
  Its published replacement is now adopted without a local override.
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

## Published compiler upgrade (78d92a7)

Five local fixes are retired in favor of published implementations: #1481 at
91c41cd9a, #1752 at 6aa792201, #1763 at 07d97ef66, #1775 at a02f938b0 and
#1778 at 75abfed4b. Their old lowering, schema, binding-identity annotations,
regressions and mutation overrides are removed from the downstream stack.
Only #1779 remains. The preserved older results describe the earlier checkpoint
and are not a claim that a broad gate has passed on this new pin. Application regeneration, 36 app tests, 18 browser tests, compiled core tests
and four focused declarations/WebGPU regressions passed on 78d92a7.

## Published WebGPU namespace fix

Claude relayed the published #1779 fix, 7d1c48f3c, through the local agent hub.
Adopted latest published main `27e3f76eceecd50d3b2d839ecc8b22f3842a926c`, including that fix. The final patch,
patch-application script and obsolete patch-only regression are retired. The
private compiler checkout is clean. Regeneration, all 38 app tests, all 23
production browser tests and 24 compiled core tests pass on this published pin.

## Bloom and retained presentation checkpoint

The Rust dark postprocess graph now performs threshold extraction, all four
Gaussian pyramid levels, exact raw/bilinear-paired axis selection, the dedicated
particle-light area downsample, near/far scale composition, ACES/sRGB presentation
and cached display reveal. Low quality preserves its two-level layout and 0.15
bloom strength. Device-enabled packed storage selects rg11b10ufloat for visible
levels only; rgba16float remains the fallback and particle-light format.

Four browser comparisons cover high/low quality with the packed feature enabled
when supported and with it deliberately disabled. Every intermediate horizontal
and vertical target, retained presentation and final display has zero differing
Float32 readback channels compared with actual upstream effects and independent
upstream uniform functions. The input includes a narrow HDR ray and gradients;
nonzero/finite assertions prevent blank-output false positives. Reveal values
0, 0.25 and 1 match while the cached presentation texture retains identity.

A fifth browser test rejects the fourth native pipeline promise during graph
construction and verifies all partially created textures and uniform buffers are
destroyed exactly once. The renderer catches native rejection and returns the
error while preserving Rust's Result API. Shader inputs cross one generic
type-erasure boundary; core math remains independent of JS bindings.

Outstanding compiler diagnostic: [#1784](https://github.com/rust-js-lang/rust-js/issues/1784)
is a minimal confirmed false-positive field-liveness warning through generic
js::any_of. It is also sent to Claude through agent-hub for upstream support.
Current JavaScript and GPU behavior are correct; strict -D warnings fails this
probe. The new code has no allow attribute or source workaround.

Particles, light-theme rendering, runtime/lifecycle integration, the full page
and public deployment remain required. The broad compiler gate history above
is not replaced by a claim that the complete gate passed on this pin.

## Retained runtime and particles checkpoint

Rust now owns the complete typed numeric control normalization, shared range
catalog, retained camera/framing/optical state and native GPU runtime resources.
Normalization compares every numeric field against upstream across finite,
out-of-range, NaN and infinite inputs and preserves the caller's records. The
complete Rust schema is used by application callers; this is not a claim of a
complete JavaScript SDK or historical Fast Refresh object compatibility API.

Independent upstream state transitions compare responsive fit, viewport epsilon,
orbit, lamp aim, resize, camera controls and presentation-only changes. GPU
Float32 matrices and every resulting spectral upload match exactly; ideal f64
basis vectors use the existing tight camera tolerance. Native and compiled Rust
also prove whole-orbit rounded-prism containment and unchanged-state identities.

The native runtime reserves the original high-tier vertex capacity, reuses its
GPU buffer and scratch storage across updates and quality changes, and creates
wireframes lazily. Studio/debug preparation returns the same pending, fulfilled
or rejected promise. An injected studio failure waits for a deliberately held
debug bake to complete before propagation. Every owned and temporary GPU texture
and buffer is checked to be destroyed exactly once on cleanup.

The 2,200-instance dust pass uses the original unchanged deterministic WGSL.
Actual upstream draws match every output byte at times 0, 3.2, 9.7 and 1,000,000,
and with zero/partial/full reveal. Distinct animation checksums prove time is
effective; zero reveal renders no dust. Animation retains the camera matrix and
allocates no new GPU uniform buffer.

All 40 app tests, 27 production browser tests and 26 native/compiled Rust tests
pass. Default rust-js-aware Clippy exits successfully; only #1784's four known
false-positive warnings remain, with no suppression. Clippy's fixed-size chunk
suggestion uses idiomatic as_chunks successfully on the published compiler.

Generated lampAt gains one conservative struct copy after normalization makes
that struct mutable elsewhere. Behavior remains correct; the output-precision
observation is sent to the upstream support agent. No source workaround applied.

Still required: assemble the retained draws into both complete scene graphs,
light-theme materials/shadows, the canvas/frame-loop and React lifecycle, the
full landing content, production build, local completion gates and public Pages
deployment. This checkpoint does not complete the landing milestone.
