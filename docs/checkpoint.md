# Output cards checkpoint

This is an unfinished development checkpoint, not a deployable landing-page release.

## Included work

- Rust CI output animation, deferred images, video playback/progress, and the four-card output section.
- Original image and video assets, with build-time responsive AVIF/WebP/PNG generation.
- Browser comparison fixtures and initial widget tests.
- Published rust-js revision 618662df715d2af347987d76ca8525fd7ab66f39, replacing the downstream Battery API patch.

## Verification and remaining failures

At this checkpoint, static media tests pass: 11 tests and 336 assertions, including byte comparisons against the pinned native Next image encoder. git diff --check passes.

The full Rust web build is blocked by imported JSX props flattening: https://github.com/rust-js-lang/rust-js/issues/1794. The failing compiler regression is retained in patches/rust-js/regressions/jsx-imported-props.test.ts. No compiler workaround is applied.

Generated widget JavaScript predates the final Rust changes and must be regenerated after that compiler fix. The four-card section has no generated output yet. Do not treat the checked-in generated files as proof of a successful current build.

All seven focused output-widget browser comparisons now pass. They cover CI timing and viewport pause/resume, reduced motion, video metadata/time formatting, actual retained video playback, mobile/desktop image activation with exact decoded pixel comparisons against the native Next optimizer, and missing IntersectionObserver.

The earlier CI timeout was an image exception unmounting the page: the catalog covered deviceSizes but omitted imageSizes needed by 34vw. Asset generation now uses both native size lists and asks getImageProps for its additional development loader-probe widths; 144 derivatives are generated. Native getImageProps coverage prevents this omission recurring. CI mock visibility changes now target only its own observer.

Image activation also exposed the CJS default-export ambiguity of next/image under Vite. The fixture uses Next's shipped ESM image entry, with no component shim; the production build must use that same vendor entry. Both implementations pass the exact pixel tests with this entry.

These browser checks exercise the checked-in generated widgets described above, not a successful regeneration of the complete current Rust crate. The full build blocker and stale video output still require resolution. The previously recorded intermittent light GPU comparison also remains unresolved.

The complete page shell, remaining landing sections, final parity checks, and GitHub Pages deployment remain outstanding. See implementation.md for the milestone requirements.
