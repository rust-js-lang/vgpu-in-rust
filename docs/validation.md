# Validation evidence

The complete landing page is not deployed. These checks prove scene mathematics,
React leaves and GPU building blocks; they do not prove a completed landing page.

## Current checkpoint

The published rust-js gitlink is
`c4aa4e5d54028aa834b0892838e22d3cb7bf5649` (2026-11-14), including the published
#1784 field-liveness and #1785 function-pointer cast fixes. One minimal downstream
patch for the neighboring #1786 classification bug remains;
see patches/rust-js. No private compiler commit is the gitlink.
The upstream nullable-promise, async-declaration, basic derived-Debug and native
Error-family fixes are used unchanged.

Passing application checks:

- 28 native Rust tests and the same 28 compiled JavaScript tests.
- 48 app tests with 161,395 assertions: scene optics, framing,
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
- 55 production-built Chromium checks. Ten cover setup-tab parity, copy/reset,
  rapid transitions, layout stability, unavailable/denied clipboard, reduced
  motion, mobile sizing and unmount cleanup. Forty-five require a real WebGPU adapter:
  all 44 retained shaders compile; malformed shader diagnostics work; the Rust
  fullscreen pass copies exact pixels/orientation; the additive light pass
  renders red/blue spectral output and disappears with zero reveal; both glass
  interface pipelines compile for single-sample and 4x MSAA targets. Connected composition is now also checked, as detailed below.
- Native core Clippy previously passed. The current rust-js-aware web Clippy
  passes with -D warnings after adopting the published #1784 fix, without
  suppression or a visibility workaround. LazyLock declarations use the
  published opaque @rust-js/std type.

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

Finish quality policy and canvas resource lifecycle. Both assembled scene graphs
are already GPU-verified. Finish the full
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

## Assembled dark scene checkpoint

The dark graph now connects the actual rounded geometry, exterior/internal
spectral light, two-interface environment glass, MSAA scene, bloom pyramid,
display presentation and 2,200 animated dust instances. Its stable production
backdrop is recorded in a native GPURenderBundle. Back, wall and caustic views
retain the source's direct ordering and suppress the glass/dust/bloom layers
where required. Both optional wireframe draws compile lazily and preserve their
original line/triangle topology and premultiplied blending.

BoundPipeline is the single owner for reflected native group construction,
retained uniforms, rebinding and cleanup across fullscreen, particle and mesh
passes. Allocations owns construction rollback for both the full graph and its
postprocess graph. Indexed prism encoding belongs to geometry and serves glass
and wireframes; the older glass-only encoder is removed. Strict hex parsing has
one shared owner for scene uniforms and sRGB wall clearing.

Four full-pipeline browser comparisons use the original createDarkPipeline and
runtime implementation on the same native device. High/low quality and packed
bloom (when enabled by the device) match every byte of backdrop, scene,
presentation and final output. The input exercises nonblack wall color, pointer
orbit, lamp target, partial beam aperture, cached time/reveal frames and all
three wireframe combinations. RGB-only assertions reject blank frames, and zero
display reveal is explicitly checked to be black.

Portrait resizing updates target storage and reflected texture groups while
retaining every compiled pipeline, uniform buffer and backdrop bundle. No shader
pipeline is compiled during the Rust resize. A frame requesting cached rendering
after resize still rebuilds the invalidated scene and matches upstream pixels.

A late failure injected into dust pipeline creation releases every texture and
buffer made by the full graph exactly once, including the already prepared
postprocess resources. The shared runtime survives and retries successfully
without rebaking its environment. All 41 app tests and 32 production browser
tests pass; native and compiled scene invariants remain 26 each. Clippy reports
only the existing #1784 false-positive warnings. No compiler patch was needed.

The light-theme graph/materials, canvas/frame-loop/React lifecycle, complete
landing content and Pages deployment still remain. These real GPU comparisons
prove the dark scene, not a deployed or finished landing page.

## Light material and cast-shadow checkpoint

The light-theme foundation now includes the source's procedural wall normals,
wall lighting, spectral caustic profile, CPU recovery generation, rounded mip
chain and authored 768x512 global-light mask. Compute baking uses native WebGPU
pipelines, with the original synchronous API fallback and bitmap decode retry.
The shared preload retains promise identity and clears failed fetches for retry.
All generated material levels match actual upstream GPU or CPU bytes.

Ten material browser checks exercise all 31 mip levels per three-texture set,
missing createImageBitmap, wrong decoded dimensions, rejected decoder options,
missing async compute creation and rejected compute pipelines. Failure injection
checks submission, upload and generation cleanup. The generation rejection test
first exposed a port bug: generation outside its promise bypassed wait-for-all
cleanup. Moving generation into the owned task fixes the leak; all cases pass.
Bitmap and temporary/partial texture destruction are checked explicitly.

Swept convex-hull shadow geometry, penumbra rings, u32 topology and shader
uniforms match upstream. Two additional real-GPU tests compare the native shadow
and front-facing premultiplied glass accent at three camera angles each. Every
readback channel matches; changed RGB assertions reject empty draws. The glass
pipeline now takes the explicit Back/Front/Accent layer instead of coupling
blend and culling to a boolean. Fullscreen pipeline compatibility is a format
and sample-count signature, allowing direct canvas presentation without an
unneeded offscreen allocation. The dark suite passes after both API migrations.

The compiler's #1785 patch shares function-origin resolution between standalone
and tuple-path function-pointer feature checks. Its 19 focused tests (142
assertions), 88 architecture/catalog tests (44,443 assertions), 118 native
compiler tests and strict compiler Clippy pass. All five affected mutations are
caught. Patch application is proved against the published pin in a disposable
checkout. This scoped evidence does not replace the broad compiler gate above.

The current app checkpoint passes 48 app tests (161,395 assertions), 44 Chromium
checks and 28 native/compiled core tests. The light graph assembly, canvas/frame
loop, React lifecycle, complete landing content and public Pages deployment
still remain. The project is not yet a deployed landing page.

## Connected light graph and published cast fix

The complete light graph now owns its retained HDR backdrop and MSAA scene,
wall/material sampling, u32 cast-shadow draw, projected caustics, back/front
physical glass, front accent and direct display presentation. A native render
bundle preserves the source order: wall, shadow, white/outgoing caustics, back
glass, internal caustics. The source's alternate views and debug light mesh
ordering are preserved, including the complete light wireframe in wall view.

Four full-pipeline tests compare actual createLightPipeline against Rust for
high/low quality and 1x/4x scene samples. Every byte of backdrop, scene and final
output matches through all views, cached reveal frames, portrait resize, every
view/wireframe combination and all four tone mappings. The 1x test explicitly
sets the original SDK wrapper's compatibility flag; it tests that target layout
on a real WebGPU device, not a claim that this device is a compatibility adapter.
After resize the reference receives its canvas lifecycle's required scene-refresh
request; Rust invalidates its cached scene internally and is given a cached-frame
request to prove that invalidation. Pipelines, uniform buffers and the stable
backdrop bundle retain identity across resize; no pipeline compiles during it.

Three failure tests inject native pipeline, bind-group and environment failures.
An environment failure waits for a deliberately held compilation branch before
rollback. All owned textures and buffers are destroyed exactly once, and retry
reuses the shared environment. The bind-group test first exposed a leaked
presentation uniform buffer. BoundPipeline now catches a native unwind, releases
its newly allocated uniforms and rethrows the original payload. This shared
owner protects every draw; it does not translate or swallow the native failure.

Claude relayed the published #1785 fix at c4aa4e5d5. Its old patch is removed.
The broader retained regression caught a separate incomplete classification:
is_function of a known namespaced binding cast was treated as a caller value,
omitting its missing-root guard and throwing ReferenceError. A known ordinary
Rust function cast was also accepted inconsistently. Filed #1786 and shared the
minimal reproducer and correction through agent-hub. The new patch makes
asks_of_value consume the existing cast_from owner; it adds no second origin
analysis. Twenty focused tests (156 assertions) pass; both affected mutations
are caught. Their cold post-upgrade run averaged 43 s each (13.4 s build and
4.6 s tests per mutant, plus cold baseline preparation); the compiler upgrade
itself rebuilt in 45.66 s. Patch application passes 21 exact checks against
the published pin. The broad compiler gate remains outstanding.

The complete local app suite passes 51 browser checks, 48 app tests (161,395
assertions, including patch proof), 28 native/compiled core tests and strict
web Clippy. The full landing page, canvas interaction/lifecycle, quality policy
and public Pages deployment remain unfinished.

## Serialized theme and quality activation

Graph preparation now consumes GraphInputs: an owned snapshot of camera/control
values and borrowed native GPU handles, plus the shared environment-ready promise.
It holds no mutable live-runtime borrow across an await. A candidate's mesh layout
is captured independently; only activation uploads that layout to the shared mesh.
This closes an integration gap in the isolated constructors, which previously
changed the live layout before preparation could succeed or become stale.

PipelineController serializes requests, retains the current graph while a
candidate prepares, discards stale candidates, keeps the initial ready promise
identity and clears settled tasks for retry. Disposal immediately drops the
active graph and returns a stable cleanup promise while pending preparation
still needs the runtime. Quality mesh dimensions and runtime capacity now consume
the same PipelineQuality owner. Rust owns these behaviors; no application
TypeScript is introduced.

Four new real-GPU checks exercise this boundary. A delayed light pipeline leaves
the dark scene interactive: pointer orbit changes still render and every output
byte matches the original controller. Rapid theme/quality reversals activate only
the latest requested pair; event order matches upstream. Separate quality/theme
failures preserve the active image and high-tier mesh byte for byte, then retry
successfully. Disposal during delayed light preparation waits, activates nothing
and releases every allocated texture and buffer exactly once after runtime cleanup.

All 55 browser checks pass (16.1 s), alongside 48 app checks with 161,395
assertions, 28 native and compiled core invariants, and strict web Clippy. The
current pin's attempted broad compiler gate was stopped to avoid competing with
an earlier shared-machine gate; it is not counted as passing. Scoped app evidence
supports this app checkpoint only. The full canvas/frame loop, DOM activity and
responsive observers, automatic quality policy, React background/root, remaining
landing sections and Pages deployment still remain.
