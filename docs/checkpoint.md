# Output cards checkpoint

This is an unfinished development checkpoint, not a deployable landing-page release.

## Included work

- Rust CI output animation, deferred images, video playback/progress, and the four-card output section.
- Original image and video assets, with build-time responsive AVIF/WebP/PNG generation.
- Browser comparison fixtures and initial widget tests.
- Published rust-js revision 618662df715d2af347987d76ca8525fd7ab66f39, replacing the downstream Battery API patch.

## Verification and remaining failures

At this checkpoint, static media tests pass: 10 tests and 86 assertions, including byte comparisons against the pinned native Next image encoder. git diff --check passes.

The full Rust web build is blocked by imported JSX props flattening: https://github.com/rust-js-lang/rust-js/issues/1794. The failing compiler regression is retained in patches/rust-js/regressions/jsx-imported-props.test.ts. No compiler workaround is applied.

Generated widget JavaScript predates the final Rust changes and must be regenerated after that compiler fix. The four-card section has no generated output yet. Do not treat the checked-in generated files as proof of a successful current build.

The initial media metadata/time-format browser comparison passed. The CI animation comparison timed out and still needs diagnosis. Image activation and real video playback coverage remain incomplete. The previously recorded intermittent light GPU comparison also remains unresolved.

The complete page shell, remaining landing sections, final parity checks, and GitHub Pages deployment remain outstanding. See implementation.md for the milestone requirements.
