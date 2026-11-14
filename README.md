# vgpu-in-rust

A faithful Rust/rust-js port of [vgpu](https://github.com/vercel-labs/vgpu), beginning with its interactive landing page. MIT licensed; original attribution is retained.

**Implementation in progress. The full landing page is not deployed.**

Rust owns the application, React components, scene calculations, WebGPU commands and resource lifecycle. The pinned rust-js submodule compiles it to JavaScript; Vite packages the browser modules. Original WGSL, CSS and media retain their native formats. Upstream application TypeScript is used only by independent reference tests.

The current port includes the real dark/light prism pipelines, responsive canvas, automatic quality controller, interactive hero/setup tabs, four output cards and the shader editor with an actual on-demand GPU ring preview. Original keyboard, visibility, deferred loading and cleanup behavior are checked against upstream. Remaining page sections, final shell/fonts, optional debug/performance surfaces and deployment are still work.

## Development

Use Bun. Clone with submodules, then run:

```sh
bun run setup
bun run compile
bun run test:native
bun run test:compiled
bun run test
bun run test:browser
```

Setup/compile apply any issue-linked compiler corrections retained under [patches/rust-js](patches/rust-js/README.md). Published upstream fixes replace these patches as they become available.

Reference comparisons require the original checkout in an adjacent `vgpu` directory, or `VGPU_UPSTREAM_DIR` pointing to it, at the revision recorded in [UPSTREAM.md](UPSTREAM.md). The suite verifies the pin. Browser tests require a real WebGPU adapter and use Chromium; a fallback image cannot satisfy GPU checks. Tests run locally; the planned Pages workflow only builds and deploys.

## Current evidence

All 66 application tests (454,329 assertions), 32 native and 32 compiled core tests, and strict rust-js-aware Clippy pass. All ten new shader editor checks and eleven output-card checks pass, including exact native-reference content/layout, media bytes, submitted GPU pixels and lifecycle failures.

The full browser run is **110/113**. Three prism GPU comparisons still differ by exact bytes and remain release blockers. No pixel tolerance or retry-to-pass is used. The complete landing milestone remains open until those discrepancies, the remaining page coverage and the live Pages verification are resolved.

[Checkpoint details](docs/checkpoint.md) · [Architecture and plan](docs/implementation.md) · [Validation history](docs/validation.md)

Live GPU checkpoint screenshots: [shader desktop](docs/screenshots/shader-demo-desktop-checkpoint.png) · [shader mobile](docs/screenshots/shader-demo-mobile-checkpoint.png) · [hero desktop](docs/screenshots/hero-desktop-checkpoint.png) · [hero mobile](docs/screenshots/hero-mobile-checkpoint.png).
