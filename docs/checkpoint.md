# Output cards checkpoint

The complete landing-page milestone is still in progress; this is not a deployment announcement.

## Published compiler

The app uses unmodified rust-js 7977d6f205090e273de0601e0425fa233ae38e9f, including the imported JSX props fix at 77a03e8e5. No active compiler patch remains. The full Rust web crate now builds, and all generated widgets and the four-card section have been regenerated. Strict rust-js-aware Clippy passes, alongside all 32 native core tests and the same 32 compiled JavaScript tests.

The upstream team received the failing case and compatibility constraints through agent-hub. Its seven imported-props tests pass locally, covering cross-crate props, aliases, required fields, privacy, Range and unrelated macros. An inactive HIR prototype requested for glob-import follow-up is retained under docs/proposals; the app does not apply it.

## Output section

Rust owns the original four-card section, CI timeline, deferred images and video. Original PNG, MP4 and SVG assets are retained. Static AVIF/WebP/PNG derivatives call the pinned native Next optimizer directly, including its geometry, encoders and concurrency policy, with the exact upstream-pinned Sharp build. Concurrency is fingerprinted because changing it changed AVIF bytes and decoded pixels in a controlled probe. The lockfile override prevents Next silently installing another native codec version; a test checks both resolve the same package and codec versions. Build-time asset generation is hash-validated and ignored by Git.

All 65 app tests pass (454,327 assertions), including 13 static-media checks and independent byte comparisons with upstream raw assets. Meaningful browser checks cover CI timing and viewport pause/resume, reduced motion, video metadata/time formatting, actual retained video playback, responsive image activation and missing IntersectionObserver. Assembled light/dark mobile/desktop comparisons check normalized markup, every rendered element’s exact computed CSS and physical geometry, and selected image bytes against the native Next optimizer.

Whole-page screenshot equality was tested and rejected as an invalid oracle after unchanged upstream-versus-upstream controls also differed: one control changed 150 color channels by at most 2 around rounded card edges. Software CSS rasterization and software/sRGB rendering did not fix the control. The assertions use exact semantic values and bytes, with screenshots retained for human review; no pixel tolerance or per-pixel mixing is used. Native Typed OM records computed CSS; legacy getComputedStyle exposed unstable used values for the fixture’s auto margins even with identical geometry. Real WebGPU comparisons retain their separate strict oracle and earlier unresolved investigation.

All eleven output-card browser checks pass. Reference encodes are cached by input URL, and the codec-heavy file runs in order beside the parallel GPU suite to avoid oversubscription without changing native encoder settings. The final complete browser run passes 102/103 in 35.5 seconds; the remaining low-quality packed dark graph differs by 52 background bytes in wire-false-true. No GPU assertion was relaxed and the earlier GPU release blocker remains. The complete page shell, remaining sections/live demos, fonts, final full-page comparison, optional debug/performance surfaces and GitHub Pages deployment remain outstanding.
