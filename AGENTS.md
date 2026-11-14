# Working on vgpu-in-rust

Use idiomatic Rust for application and scene behavior, compiled with the pinned rust-js submodule. Use Bun for JavaScript tooling and GitHub CLI for GitHub operations. Work on main; commit messages use feat(vgpu), fix(vgpu), chore(vgpu), or perf(vgpu), and include Co-Authored-By: Codex <codex@openai.com>.

Read UPSTREAM.md and docs/implementation.md before porting. Keep the upstream source checkout read-only. Preserve the landing page's actual interactive WebGPU behavior, content, responsive layout, and light/dark themes. Images, CSS and WGSL may retain their original formats. Do not hide application TypeScript behind a Rust wrapper or replace the prism with a static screenshot.

When idiomatic Rust exposes a rust-js gap, write a failing regression, file a generic upstream issue, patch this project's compiler submodule, and store the patch and issue link under patches/rust-js. Keep its published gitlink unchanged unless intentionally upgrading. Never change another session's compiler checkout.

Run native and compiled-Rust tests plus direct comparisons with the pinned local upstream source. Browser tests must exercise real WebGPU output and interaction as well as fallback, responsiveness and lifecycle. All tests run locally. The Pages workflow only builds and deploys, using current action releases pinned to full commit hashes.

Never mark the landing milestone complete merely because tests or deployment are green. Verify the actual public page against upstream and audit the requirements in docs/implementation.md.
