# Validation evidence

Implementation is in progress. No public landing page is deployed yet. The final full compiler gate remains required before publishing the checkpoint.

## Passing app checks

The Rust scene crate implements pointer easing, responsive projection framing, rounded/beveled prism geometry, wireframe extraction, optical planes, intersection, Snell refraction, total internal reflection, Fresnel transmission and CIE/D65 spectral mapping. React leaves include the wordmark, inline literals and memoized hero-tab context.

- Fourteen native Rust scene tests pass; the same fourteen pass after rust-js compilation under Bun.
- Fourteen app comparison/reproducibility tests pass with 19,574 assertions: nine optical/interaction comparisons, exact Float32 mesh vertices and topology across fifteen shape/depth combinations, three component/context comparisons, and clean application of all eight compiler patches.
- The patch audit also rejects compiler edits omitted from the tracked stack. Generated browser modules match their Rust source, and the new web code passes rust-js-aware Clippy with warnings denied.
- The 44 retained shader modules match upstream byte for byte. They are not yet connected to a renderer.

These checks do not prove a rendered GPU scene or complete landing-page fidelity.

## Compiler fixes

- #1748: strict TypeScript rejects wrong Cow payloads; runtime variants/payloads remain intact. Thirty-five neighboring declaration tests passed and both mutations were caught.
- #1481: the Cow AsRef/Borrow corpus agrees with native Rust across strings, slices, arrays, paths and both variants. Two meaningful dictionary mutations were caught.
- #1751: metadata-only Node/Bun runner switches invalidate old artifacts; emitted imports and actual execution match. The common settings-input mutation is caught.
- #1749: the complete metamorphic sample is partitioned without missing or duplicate parts; all 48 CI-planning tests passed.
- #1752: strict lazy-state types agree with actual Rust-generated initialization. Pending/Ready is an enum. Both declaration mutations are caught; an unbatchable mutation site was moved to the existing batchable function-body catalog after its original 120-second limit was reached.
- #1754: both memory-cap assertions pass after removing the fixture's redundant byte count, retaining allocation and sleep.
- #1762: CPU and memory backfilling probes failed before the fix. A third bounded probe reproduced fractional accounting residue blocking an empty pool. Active reservations now own accounting. All three probes and the 51-test planning/admission group pass; no timeout or validation coverage was relaxed.
- #1763: strict TypeScript preserves five React handle forms and rejects wrong payloads. Runtime callback/ref identity is unchanged. React 18 uses MutableRefObject and React 19 uses RefObject; both version-selection tests pass. The metadata-reader mutation is caught. A guide preflight matched all 402 examples.

## Broad compiler checks

No interrupted or failing run is counted as passing. The first run reached its enforced 720-second limit with a serial metamorphic tail. A later run exposed an order-dependent Cargo assertion and the slow memory fixture; both have focused fixes. Another full run finished in 546 seconds with one architecture failure, the lazy schema's loose flags, subsequently replaced by an enum.

The next run reached 720 seconds with suites still starting. A backfilling rerun then passed the corpus, browser suites, matrix samples, lint, compiler unit tests and changed mutations, but remained idle with no child processes; it was stopped. The independently reproduced fractional reservation defect is now fixed. The next frozen run also reached 720 seconds without a final suite result. Completed stages passed; a post-run sample still saw the occupied machine 99.4% busy on fourteen cores, with load near eighteen and unrelated work active. The cost is recorded in the compiler development ledger. The next bounded attempt needs live pending-job diagnostics and an exclusive quiet window. A full passing result remains unproven.

These timings were observed alongside other local work and are not controlled performance comparisons.

## Remaining milestone gates

Port and render the actual GPU scene and all landing sections; add meaningful Playwright desktop/mobile, theme, interaction and lifecycle checks; compare against upstream; publish the source and configure the build/deploy-only Pages workflow; verify the actual deployed page. Keep the milestone open until those gates pass.
