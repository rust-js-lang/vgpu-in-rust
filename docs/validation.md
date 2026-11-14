# Validation evidence

The complete landing page is not deployed. These checks prove scene mathematics,
React leaves and GPU building blocks; they do not prove a completed landing page.

## Current checkpoint

The published rust-js gitlink is `3ba28b44e2ccac37a6ee61dcabffd573bf6d5a33` (2026-11-14), including vendor CSSProperties fix #1806. A tracked correction for JSX whitespace/child identity (#1807) remains until upstream publication. See docs/checkpoint.md for the current shader editor proof and exact remaining release gates. The full browser run is 110/113 in 40.7s; all ten new shader-demo checks pass, while three prism GPU comparisons still fail. Strict rust-js Clippy and 32 native/32 compiled core tests pass. Results below are historical checkpoints unless explicitly updated.

The upstream nullable-promise, async-declaration, basic derived-Debug and native
Error-family fixes are used unchanged.

Application evidence:

- 32 native Rust tests and the same 32 compiled JavaScript tests.
- 53 app tests with 453,992 assertions: scene optics, framing,
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
- The latest full run passes 91/92 production-built Chromium checks. The
  intermittent exact light-graph comparison fails; details are recorded below.
  Ten cover setup-tab parity, copy/reset,
  rapid transitions, layout stability, unavailable/denied clipboard, reduced
  motion, mobile sizing and unmount cleanup. Sixty-eight require a real WebGPU adapter:
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

Finish React canvas integration and optional debug/performance surfaces. Both assembled scene graphs
are already GPU-verified. Finish the full
landing root and every section. Compare full-page GPU output, themes,
desktop/mobile framing, interaction, fallback and cleanup against upstream.
Preserve the compiler gate history without claiming a passing broad run. Source checkpoints are published. Enable
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

## Native canvas and browser lifecycle checkpoint

BrowserRenderer now connects actual swapchain output to both retained graphs.
It owns device negotiation, optional packed-bloom feature fallback, surface
configuration, pointer listeners/easing, mobile automatic motion, responsive
framing/DPR measurement, coalesced resize work, visibility/offscreen observers,
30/60/90 FPS caps, reveal timing and deferred disposal. The clock starts at the
first environment frame, matching the original frame service rather than being
restarted after asynchronous preparation. Capability selection has one owner
shared by device negotiation and live bloom format decisions.

Four canvas-level comparisons use the actual original browser renderer with
controlled clock/rAF input on real WebGPU devices. Desktop/mobile and normal/
reduced-motion cases match every captured swapchain byte through reveal, pointer
motion, responsive canvas/slot resizing, theme switching, forced Low quality and
DPR reduction. Settled desktop light scenes stop submitting frames; mobile
automatic motion continues under its cap. A separate native-rAF test verifies
ordinary scheduling and theme switching.

Eight lifecycle checks cover absent WebGPU (including a throwing reporter),
unavailable canvas context, configure failure with original error identity and
canvas-size restoration, optional-feature rejection/fallback, disposal during
a pending device request, visibility/offscreen pause/resume and disposal during
delayed GPU preparation. Every tracked texture and buffer is destroyed exactly
once in the delayed-unmount case; context/device cleanup happens once and no
commands are submitted after device destruction.

The former default headless shell failed even a minimal plain-JavaScript native
swapchain clear with device loss: "A valid external Instance reference no longer
exists." Retaining GPU/adapter objects and removing controlled timing did not
fix it. Full Chromium passes that same control and the renderer cases, so the
Playwright configuration explicitly uses channel chromium. No app rendering
fallback or weakened GPU requirement masks the failure.

Full Chromium also exposed two test-oracle issues. Hero-tab transition tests
now pause the fake clock explicitly, so action execution cannot consume the
150 ms outgoing-layer interval. The original GPU renderer itself can produce
two outputs for an identical draw: a recorded case changed 44 bytes on the
reference redraw while the Rust result stayed stable and then matched it exactly.
Prism uploads (17,328 bytes), spectral vertices, camera values and all f32-packed
uniform inputs were identical. Light comparisons therefore capture all three
attachments before another draw and require an exact whole-frame match against
one of at most two native reference observations. A second observation is
allowed only for a fresh scene draw with identical inputs. It never mixes
pixels from observations or applies an epsilon. Observation details are attached
to test results. Cached-scene comparisons retain their original strict check.

Claude relayed #1786 at published 4c58cdcf6. Its implementation replaces the
last patch and patch-application helper; the private compiler checkout is clean.
Both the published suite and the temporarily restored broader port regression
pass: 20 tests, 159 assertions. The temporary regression is removed afterward.
Application regeneration takes 28.93 s including the compiler rebuild.

Current evidence: all 69 browser checks pass in full Chromium (28.1 s), 47 app
tests pass with 161,374 assertions, all 28 compiled core invariants pass, native
core tests pass, and strict web Clippy passes. The older broad compiler gate is
still historical, not a claim about this unmodified published compiler pin.
Automatic GPU-tier/battery/runtime-health quality policy, React canvas/theme
integration, the complete landing content, production build and public Pages
deployment remain required.

## Automatic quality policy core and open GPU comparison gate

The pure frame-health monitor is ported from the pinned original: refresh-rate
estimation, active two-second windows, workload-specific caps, inactivity resets
and sticky downgrades. Source comparisons cover refresh rates, intentional frame
skips, invalid deltas, reset timing and deterministic jitter (291,648 assertions).
GPU-tier/battery predicates match original boundary decisions. These modules are
not yet connected to BrowserRenderer; lazy advisory loading and cancellation,
quality-state subscriptions and automatic transitions remain work.

Native and compiled Rust now pass 32 tests; the app suite passes 51 tests and
453,164 assertions. Strict rust-js Clippy passes. Plain cargo clippy was also
tried: it reports JSX-only names as unused, as the guide documents; the canonical
rust-js Clippy command checks their actual uses and passes in 7.56 s.

The optional Battery Status API was absent from both the published pin and a
newer local upstream checkout. A temporary native Rust probe failed with E0599.
Issue #1790 records it. The app-private generator includes the battery-status
spec and BatteryManager interface, retaining canonical promise/getter/event
generation. One compiler regression passes (10 assertions); removing
Navigator.get_battery makes it fail, and restoring the patch passes. The tracked
patch includes that regression and the decision/guide amendment. A clean-tree
application/reversal test passes. No broad compiler gate is claimed for this
downstream patch. The published gitlink remains 4c58cdcf6.

The latest complete browser run passes 67/69 in 30.3 s. High-quality dark graph
(fallback bloom) differs in the wireframe background by 63 bytes. High-quality
light graph differs in back-view wireframe background by 44 bytes, with 20
output bytes differing. A repeated graph-only diagnostic run passes 23/24
cases and reproduces the light difference with single-sample targets too.
Packed uniform values, prism upload bytes and spectral vertices match. In this
new observation both the Rust and original repeated frames are individually
stable while still differing from one another (44/44/20 attachment bytes).
This is stronger contrary evidence than the earlier reference-variation case:
the earlier bounded reference redraw does not explain or accept this failure.
No tolerance, retry-to-pass rule or relaxed assertion was added. Byte-level and
self-comparison diagnostics now accompany the existing strict failures. GPU
pipeline descriptors, emitted shaders and bound resources need comparison
before attributing this to driver behavior or changing the oracle.

These failures remain an explicit landing-page release gate. The pure policy
checkpoint is published independently; no production deployment is claimed.

## Live automatic quality integration checkpoint

BrowserRenderer now defers the Auto controller until after a successful High
frame, another rAF and a zero-delay task. The controller and vendor detector
remain two distinct dynamic imports in generated JavaScript. Forced preference
changes and disposal cancel scheduled work and invalidate older in-flight
factories. Auto health accounts for skipped capped frames, ambient dust, mobile
workloads, inactivity and resize/theme/quality resets. Quality state reports
preference, effective pipeline and reason through listeners with native Set
identity/removal semantics. Failed Low preparation restores the live High scene
and DPR, reports once and remains recoverable.

The advisory controller preserves the original Promise chains, including
ordering of synchronous probe failures and removal of battery listeners after
an immediate initial downgrade. Ten browser comparisons match actual upstream
GPU/battery signals, failure logging, runtime health/reset, late disposal and
listener identity. Six real-GPU renderer cases cover deferred loading, stale
factory results, stale downgrade callbacks, import rejection/throw/retry,
successful Low activation and failed Low preparation. A separate default-path
case loads the actual 6.0.20 vendor detector after production frames, verifies
same-origin benchmark requests and checks the 16-file asset manifest. Existing
canvas parity cases now also compare quality state with upstream.

The benchmark staging script retains the upstream inventory and hashes, takes
the exact version from package.json and stages the pinned vendor tables as
same-origin public assets during setup/compile. Its --check mode passes. These
are generated data, ignored by Git; production builds must run setup/compile.
The benchmark URL is page-relative so a GitHub Pages base path is preserved.
Vendor tier types include SSR and BENCHMARK_FETCH_FAILED as declared by the
installed library; the pure predicate comparisons include them.

Current checks: 86/86 browser tests pass (21.7 s), 51 app tests with 453,218
assertions pass, strict rust-js Clippy passes (5.89 s), and native/compiled core
tests pass (32 each). No pixel tolerances or retries were introduced: the
earlier GPU graph mismatch remains intermittent and is not claimed fixed by
this successful run. Full React root/content and optional debug/performance
integration, production build and Pages verification still remain.

## React landing hero checkpoint

Hero now retains the original markup, responsive classes, SVG wordmark, setup
tabs and button variants. The button uses the same pinned CVA, Tailwind merge
and Radix Slot implementations; 768 comparisons match original classes and SSR
markup across variants, sizes, custom classes, forwarded attributes and Slot
children. Six retained theme/application CSS files match upstream byte-for-byte.
The original dependencies are pinned to their upstream lockfile versions.

PrismBackground connects the compiled renderer to a React canvas effect,
framing-element measurement and document class MutationObserver. The normal
path selects the same dark/light wall colors, preloads light assets, preserves
the live scene through theme preparation and disconnects/disposes on unmount.
The optional debug graph, performance browser API and source-style per-theme
module preloading remain port work; they are not claimed implemented here.

Six browser cases pass: disabled canvas makes no adapter request; unavailable
WebGPU preserves useful hero content/tabs and reports initialization once;
desktop/mobile with initial dark/light all render real GPU output, follow a
subsequent site-theme change and clean up. StrictMode exercises mount/unmount
replay. Tracked devices are destroyed once, no commands follow destruction,
and document-theme observers disconnect once. Pixel samples are copied at
actual canvas submission: reading getCurrentTexture on a later idle rAF can
return an empty fresh swapchain image even when the compositor correctly
retains the last presented frame. The test no longer mistakes that for a
blank image. The initial test also assumed wrong CLI/MCP copy; assertions now
use the original exact accessible labels.

The acceptance screenshots in docs/screenshots show the actual Rust GPU
renderer inside the hero on desktop/mobile. The full navigation, remaining
sections/live demos, footer, final font loading, optional debug/performance
surfaces and production build/Pages deployment are still incomplete. Existing
documentation links remain on the canonical upstream site, as in the earlier
DocsLinksSection decision, since a documentation server is outside this Pages
milestone.

Current evidence: 53 app tests pass (453,992 assertions), strict rust-js Clippy
passes, and all six hero checks pass. The full browser run passes 91/92 in
14.2 s. The remaining failure is the high-quality single-sample light graph at
tone-aces: five output bytes differ. Recorded packed input differences remain
zero; the repeated actual frame changes 45/44/5 attachment bytes and then
matches reference, while the reference redraw stays stable. This adds another
observation to the unresolved GPU comparison investigation. No assertion was
relaxed and no passing full browser gate is claimed. Core Rust was unchanged
from the prior checkpoint’s 32 native/compiled checks.

## Published imported-props fix and output section

Adopted unmodified published 7977d6f205090e273de0601e0425fa233ae38e9f, including
#1794 at 77a03e8e5. The full web crate builds, strict rust-js Clippy passes
(7.44 s), 65 app tests pass with 454,327 assertions, and all seven upstream
imported-props regressions pass. No active compiler patch remains. The HIR
prototype is retained only as an inactive review proposal requested by the team.

All eleven output-widget/assembled-section browser checks pass. Desktop/mobile
light/dark comparisons check every rendered element’s exact Typed OM CSS and
geometry, normalized markup and selected media bytes against native Next.
Real MP4 playback, deferred asset activation, missing-observer fallback, CI
timing/pause/resume and reduced motion are exercised. Original raw media is
compared independently against the pinned upstream checkout. Screenshots under
docs/screenshots/output-cards-* show actual media and source layout.

The static generator now calls native Next optimizeImage directly. A controlled
concurrency=1 experiment changed AVIF bytes and pixels, so it was reverted:
production settings and their concurrency come from Next’s getSharp owner and
are fingerprinted. The original test backend duplicated encoding and launched
multiple half-core codecs beside GPU workers; caching reference results and
running that file in order eliminated its full-suite timeouts. No test timeout
was increased. An upstream-versus-upstream full-page screenshot control itself
differed by 150 channels (maximum 2) around rounded edges; software/sRGB
experiments did not fix it. Exact CSS, geometry, content and resource comparisons
replace that invalid bitmap oracle; original GPU comparisons are unchanged.

The final complete browser run passes 102/103 in 35.5 seconds. Low-quality packed
dark graph wire-false-true has 52 differing background bytes. This remains part
of the existing unresolved GPU comparison gate, not an accepted deviation or a
claim that the whole browser suite passed. Full page content/root/fonts and
Pages deployment still remain.
