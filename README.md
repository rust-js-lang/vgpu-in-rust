# vgpu-in-rust

A Rust/rust-js port of [vgpu](https://github.com/vercel-labs/vgpu), beginning with its interactive landing page. MIT licensed.

**Status: implementation in progress; the landing page has not been deployed yet.**

Rust owns application behavior and scene code. rust-js compiles the frontend to JavaScript; Vite packages it for GitHub Pages. The compiler is a pinned Git submodule, with issue-linked downstream patches when needed.

See [UPSTREAM.md](UPSTREAM.md) for source provenance and [docs/implementation.md](docs/implementation.md) for the milestone and validation gates.

## Development

Use Bun. Clone with `git clone --recurse-submodules`. Run `bun run setup`, then `bun run compile`. Native scene checks: `cargo test -p vgpu-prism`.

The current checkpoint contains scene mathematics, orbital camera/framing, rounded prism geometry, finite-width spectral light meshes with retained Float32 upload buffers, runtime control defaults and scene/glass uniforms, reflected byte packing and shader bindings, retained GPU buffers and MSAA targets, authored HDR environments with mip filtering, a retained high/low-quality bloom pyramid and display presentation, intro reveal timing, instanced dust, retained optical/camera state and GPU resource lifecycle, initial React leaves, interactive setup tabs, Rust WebGPU fullscreen effects, an additive spectral draw pass and both glass interfaces and the original shader assets. Both assembled renderers and native canvas integration are verified; automatic quality policy, the complete page and deployment remain in progress.

Run `bun run test:native` and `bun run test:compiled` for the same Rust scene invariants on both runtimes. `bun run test` also compares generated modules with the original source. For those comparisons, keep the upstream repository in an adjacent `vgpu` directory at the revision in UPSTREAM.md, or set `VGPU_UPSTREAM_DIR` to that checkout. The test suite verifies its revision before comparing.

See [docs/validation.md](docs/validation.md) for passing checks, failed attempts and remaining milestone gates.

`bun run test:browser` builds isolated Rust/upstream tab and shader fixtures. Sixty-nine Chromium checks cover tab parity/lifecycle and actual WebGPU compilation of all 44 retained shaders and exact fullscreen/spectral pixel readback. They also compare all eight HDR environment mip levels for both variants and the ordered exterior-light/back-glass/internal-light/MSAA-front composition against actual upstream rendering, and verify target lifecycle and cached preparation. Bloom tests compare every intermediate target and presentation against upstream effects, and inject a native pipeline rejection to verify partial-resource cleanup. An adapter is required; these tests do not silently pass a visual fallback. Runtime checks also cover retained vertex uploads, lazy wireframes, preparation promise identity, wait-for-all failures and complete resource cleanup. Dust pixels match upstream across long lifetimes and reveal values. The complete dark pipeline also matches upstream across view modes, packed/fallback bloom, cached frames, wireframes and portrait resizing with retained pipelines. A late construction failure cleans up completely and retries with the shared runtime. Light material baking/recovery, cast shadows and glass accents also match upstream, including mip levels and failure cleanup. The connected light pipeline now also matches the original across quality levels, target sample counts, alternate views, cached frames, tone mappings, wireframes and resize. Native canvas/lifecycle checks now pass; the full landing-page browser gate remains pending.

The retained pipeline controller now serializes theme/quality changes while the
previous scene keeps rendering. Real-GPU tests compare rapid-switch activation
and pixels with upstream, check failed-switch retry, and verify disposal waits
for pending preparation before shared runtime destruction. Native canvas DOM
observers and frame-loop integration are now verified; the full page remains in progress.

Native canvas checks now cover desktop/mobile frame timing, reduced motion,
pointer easing, responsive framing, DPR changes, theme changes, visibility and
offscreen pauses, device negotiation and cancellation/cleanup. Tests use full
Chromium (channel chromium); the headless shell loses native swapchain devices
on this machine even for a plain WebGPU clear. The React landing root and
remaining page sections are still in progress.
