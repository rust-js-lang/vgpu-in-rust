# First milestone: the interactive landing page

## Architecture

```text
Rust React components
        |
        v
Rust scene / interaction / GPU lifecycle
        |
        v
Typed browser WebGPU bindings + original WGSL/assets
        |
        v
rust-js compiler (pinned submodule) -> ES modules -> Vite -> GitHub Pages
```

Pure scene mathematics lives in crates/prism, runs natively for reference checks and is compiled by rust-js for the browser. The web crate will own React, DOM listeners, canvas lifecycle and GPU resource cleanup. Browser bindings describe real browser APIs; they must not substitute JavaScript implementations for missing compiler support.

## Implementation sequence

1. Pin upstream and rust-js, retain MIT attribution; establish native/JavaScript differential checks for prism framing, optics, camera and interaction.
2. Port the WebGPU resources and dark/light render passes, keeping shader code and scene constants faithful. Preserve pointer easing, responsive framing, quality policy, reduced motion and cleanup.
3. Port the page root, navigation, hero tabs/copy transitions, four output cards, shader editor/live ring, agent command demo, examples, documentation links and footer. Preserve original content, classes and behavior.
4. Compare generated code against each original module. Run local native and JavaScript parity checks and Playwright desktop/mobile, light/dark, interaction, GPU/no-GPU and lifecycle tests. Save visual evidence.
5. Publish rust-js-lang/vgpu-in-rust as a public MIT repository. Enable workflow-based Pages. Build/deploy-only workflow, all actions at current releases pinned to full hashes. Verify both deployment and the actual live page.

## Compiler gaps

Start with a failing regression. File a generic reproducible issue using GitHub CLI. Fix in this repository's compiler submodule and retain a tracked patch plus issue link, native comparison and relevant compiler tests/ADR/guide changes. Never alter a shared rust-js checkout or bend app code to hide a compiler gap.

## Completion evidence

A successful build alone is insufficient. Completion needs an accessible Pages URL, a public repository, the exact compiler gitlink, retained MIT notice, source-module coverage, passing meaningful parity/E2E checks, screenshots showing the live GPU scene and content on desktop/mobile, and a list of any accepted deviations. No static hero substitute. No test workflow: run tests locally.

## Renderer ownership

- crates/prism owns controls, normalized ranges, camera/framing state, optics,
  geometry values and shader parameter values. Shared color parsing lives here.
- crates/web/bindings owns reflected uniforms and BoundPipeline's retained
  native group construction/rebinding. Each draw kind owns its pipeline layout
  and fixed-function state; no second uniform-binding policy is introduced.
- crates/web/resources owns rollback inventories for partial GPU construction.
- crates/web/geometry owns indexed prism buffers and command encoding for glass
  and edge topology. The runtime owns shared geometry and environments.
- crates/web/dark_graph owns pass ordering, render bundles, cached presentation,
  resize invalidation and graph cleanup. Postprocess owns pyramid sizing,
  filtering and presentation. Graph resize retains pipelines/uniform buffers.

Tests compare the full original dark pipeline, not only isolated pass helpers.
Reference gateways export actual upstream functions exclusively and stay inside
browser test fixtures; production Rust has no upstream app TypeScript dependency.

Light assets own shared preload, native compute baking, CPU recovery and their
texture cleanup. The pure core owns procedural pixels, mip values and swept
shadow geometry. The shadow draw owns its distinct u32 indexed layout. Target
compatibility is represented by RenderSignature, independent of texture ownership;
glass fixed-function state is selected by GlassLayer (Back, Front, Accent).

The light graph owns composition, its stable native backdrop bundle, scene-cache
invalidation, view-specific draw ordering and resize rebinding. Asset baking,
environment generation and target-signature pipeline compilation begin together;
preparation waits for their outcomes before rollback or transfer of ownership.
BoundPipeline releases its allocated uniforms if native bind-group creation
unwinds, then rethrows the original payload. Both connected graph variants are
compared against actual upstream pipeline implementations in real GPU tests.

PipelineController owns requested/active theme and quality, one preparation task,
stale candidate destruction and deferred teardown. GraphInputs snapshots only
preparation values and borrows runtime GPU handles; no mutable runtime borrow
crosses asynchronous preparation. Quality layout changes are committed only at
activation, so the previous scene remains usable while another prepares.
PipelineQuality owns the shared high/low mesh dimensions and runtime capacity.

BrowserRenderer owns DOM listeners/observers, responsive measurement, pointer
easing, activity-aware scheduling, reveal/frame-rate decisions and deferred
canvas/device teardown. CanvasSurface owns device negotiation and swapchain
configuration/rollback; capabilities owns optional features and bloom formats.
EnvironmentTexture records its first frame timestamp so the browser clock keeps
the source's preparation-time origin. Scene graphs continue to own GPU passes.

FrameHealthMonitor owns active-window accounting and refresh estimation;
quality_signals owns GPU-tier and battery threshold decisions. The browser
controller consumes these decisions without rebuilding policy in DOM code.

The browser consumes those pure policies through a deferred closure-based
AutoQualityController. A generation cancels stale factories and callbacks;
quality subscriptions use a native Set to preserve listener identity and live
iteration. quality_logging owns the shared default console boundary without
eagerly loading Auto policy or the vendor detector.

PrismBackground owns the React canvas ref, document-theme observer and effect
cleanup; BrowserRenderer continues to own native GPU lifecycle. Hero retains
source layout and composes the existing tab/context/wordmark components. Button
retains the native CVA/Tailwind/Radix boundaries and forwarded DOM attributes.
The optional debug/performance React surfaces and final page shell remain work.
