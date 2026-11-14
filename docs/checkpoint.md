# Shader editor checkpoint

The complete landing page is still in progress and is not deployed.

## Compiler and team coordination

The published rust-js gitlink is `3ba28b44e2ccac37a6ee61dcabffd573bf6d5a33`. It includes the upstream vendor-style fix (#1806), which the shader poster exposed. All earlier downstream patches are retired. The remaining tracked patch is [#1807](https://github.com/rust-js-lang/rust-js/issues/1807), applied automatically by setup/compile until its upstream correctness fix is published.

Playwright exposed two wrong answers: explicit two-space JSX strings became one space, and a single edge-spaced string was split into two React children by formatting. The latter produced a measurable 1/64px text advance despite identical HTML. Both focused compiler regressions failed before the fix and pass after it. The patch preserves the exact string and child boundaries. The upstream team received the regressions and full browser evidence through agent-hub and is landing the fix; the shared compiler checkout was never modified.

The native SDK wraps pipeline failures in its VGPUError class. Rust currently retains the native cause; the landing component catches it and preserves the same unavailable/poster UI. Proper Error subclass support is tracked upstream as [#1808](https://github.com/rust-js-lang/rust-js/issues/1808). A complete SDK error-class port is not claimed. No stand-in error class or prototype workaround is introduced.

## Shader editor and actual GPU output

Rust now owns ShaderCodeScalesSection and all the original demo helpers: CodeLine, TypeScriptSource, MainShaderSource, ColorShaderSource, SourceEditor, ResultPoster and CompilerOutput. It preserves the three displayed source files, highlighted imports, original copy/classes, compiled WGSL display, responsive layout, keyboard wrapping/Home/End, focus and scroll reset. The displayed TypeScript remains example content; it does not implement application behavior.

The GPU preview uses the original main/color WGSL modules, reflected uniform packing, a real fullscreen triangle and a native canvas surface at DPR 1. It renders on demand and coalesces resize notifications into one frame; there is no idle animation loop. The renderer preserves deferred initialization, the 320px intersection margin, document visibility, fallback poster, late initialization cleanup and partial first-draw rollback. Synchronous and asynchronous fullscreen creation consume one shared descriptor builder.

All ten shader-demo Playwright checks pass in the full suite. They compare whole submitted textures against the actual upstream renderer byte-for-byte at two sizes; verify exact markup, computed CSS and geometry across mobile/desktop and light/dark; exercise keyboard behavior, no-GPU/no-observer fallback, visibility changes, late initialization after unmount, resize cancellation, failed first-draw surface/device cleanup and retained failure causes. Exact CSS states are hashed in the browser to avoid transferring megabytes into every trace; this changes transport, not the equality assertion. Live GPU screenshots at 390px and 1440px are retained under docs/screenshots and have been visually inspected.

## Verification and remaining work

Strict rust-js-aware Clippy passes. All 32 native core tests and the same 32 compiled core tests pass. All 66 application parity/asset tests pass (454,329 assertions); its new asset check also holds both preview WGSL files byte-for-byte against upstream.

The full production-built browser run is **110/113 in 40.7s**. All shader editor and output-card checks pass. The existing GPU release gate remains: high-quality dark wireframe/background comparisons differ by 54 and 63 bytes, and the high-quality 4x light ACES output differs by five bytes. Diagnostics retain exact input matches and additional whole-frame observations; no tolerance, pixel mixing, skipped assertion or retry-to-green was added. This is not evidence of a completed or production-ready landing page.

The final page shell, agent command demo, examples, fonts, optional prism debug/performance surfaces, GPU discrepancy investigation, full-page source/visual audit and GitHub Pages deployment remain outstanding. The earlier output-card checkpoint is retained in Git history; its media/encoder checks remain active and pass.
