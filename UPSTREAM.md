# Source provenance

Upstream: https://github.com/vercel-labs/vgpu

Branch: `canary` (upstream default).

Revision: `773351f8c96d1cbec28155c0e36ae3efd1cba135`. The local checkout matched the latest published default-branch commit when the port began on 2026-11-14. This is a reproducible baseline, not a claim that upstream cannot advance later.

Latest-source check: fetched `origin/canary` again on 2026-11-14; both local HEAD and the fetched upstream tip remain `773351f8c96d1cbec28155c0e36ae3efd1cba135`.

Original MIT copyright and license are retained in [LICENSE](LICENSE). Additional Rust port work is MIT licensed as well.

The first milestone follows `apps/docs/app/[lang]/(home)/page.tsx`, its `home-page.tsx`, layout, and every reachable landing component. Required prism scene modules live beneath `components/prism-background/`. Assets, CSS and WGSL remain in their original formats with attribution; application TypeScript/React must be ported to Rust, not silently retained behind a wrapper.

The complete upstream repository remains the long-term port target. Landing-page completion does not imply the entire library is ported.
