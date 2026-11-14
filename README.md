# vgpu-in-rust

A Rust/rust-js port of [vgpu](https://github.com/vercel-labs/vgpu), beginning with its interactive landing page. MIT licensed.

**Status: implementation in progress; the landing page has not been deployed yet.**

Rust owns application behavior and scene code. rust-js compiles the frontend to JavaScript; Vite packages it for GitHub Pages. The compiler is a pinned Git submodule, with issue-linked downstream patches when needed.

See [UPSTREAM.md](UPSTREAM.md) for source provenance and [docs/implementation.md](docs/implementation.md) for the milestone and validation gates.

## Development

Use Bun. Clone with `git clone --recurse-submodules`. Run `bun run setup`, then `bun run compile`. Native scene checks: `cargo test -p vgpu-prism`.

The current checkpoint contains scene mathematics, rounded prism geometry, initial React leaves and the original shader assets. The GPU renderer, complete page and deployment remain in progress.

Run `bun run test:native` and `bun run test:compiled` for the same Rust scene invariants on both runtimes. `bun run test` also compares generated modules with the original source. For those comparisons, keep the upstream repository in an adjacent `vgpu` directory at the revision in UPSTREAM.md, or set `VGPU_UPSTREAM_DIR` to that checkout. The test suite verifies its revision before comparing.

See [docs/validation.md](docs/validation.md) for passing checks, failed attempts and remaining milestone gates.
