# Validation evidence

Implementation is in progress. The complete landing page is not deployed.
The compiler now uses published rust-js `0fb07aed694681295a1f2132ee78fbf647f764ed`,
fetched on 2026-11-14. Its WebAPI promise and async declaration fixes and regressions are used unchanged.
One reproducible downstream patch retains four outstanding fixes.

## Passing application checks

- Twenty native Rust scene tests pass; the same twenty also pass after
  compilation to JavaScript. The new camera checks cover orbit/centering, matrix identity, wall containment
  and environment rotation.
- Twenty-three app tests pass with over 149,000 assertions: optics, interaction, framing,
  exact rounded mesh vertices/topology over fifteen shape/depth combinations,
  exact camera Float32 storage/matrices and orbital basis over 90 configurations,
  every spectral metadata record across high/low mesh layouts, exact Float32 light
  vertices across thirteen optical configurations, clipping, integrated flux and
  retained GPU upload/scratch identity, exact native metadata RangeErrors for
  invalid fractional/nonfinite/negative/out-of-range indices,
  exact React markup/context behavior, retained asset identity, and complete
  compiler patch reproducibility. The audit rejects unrecorded compiler edits.
- Thirteen production-built Chromium checks pass: ten compare Rust and upstream
  setup tabs, covering copy/reset, rapid switches, layout stability, denied or
  unavailable clipboard access, unmount cleanup and mobile reduced motion.
  Three require an actual WebGPU adapter: all 44 retained prism shaders compile,
  an invalid shader reports errors, and the Rust fullscreen pass copies exact
  RGBA pixels without reversing the image.
- Previously ported scene and UI modules regenerate without JavaScript changes.
  The camera preserves shared view-projection identity and matches original
  Float32 matrices exactly. A compensated norm removes the measured cancellation
  mismatch from chained two-axis hypot calls. App compilation and rust-js-aware Clippy pass.

The WGSL graph is byte-identical to the original. The fullscreen vertex stage
also matches its original shader literal exactly. Rust owns module creation,
compilation diagnostics, pipeline creation and command encoding. The existing
WGSL asset compiler resolves imports/reflection at build time. Upstream app
TypeScript is imported only by isolated reference tests.

The light mesh core reuses a packed-number scratch buffer. Its web boundary owns
the retained Float32Array and performs the original single bulk conversion;
wrong-length targets throw the original native RangeError and exact message.
Both Rust runtimes test topology ranges, finite vertices, scratch overwrite and
wavelength-subdivision flux invariance.

These checks prove components and GPU building blocks. The prism scene passes,
complete landing root and full-page visual comparison remain unfinished.

## Current compiler patch

- #1773: native RangeError constructor uses the existing Error binding convention.
  Constructor/options/declaration/throwing checks pass, and replacing it with base
  Error is caught on native class identity. All sixteen constructor tests and the
  three builtins coverage checks pass (723/723 covered members).
- #1481: Cow AsRef/Borrow uses the existing dereference owner. Its corpus case
  matches native Rust on the updated compiler, including strings, slices,
  arrays, paths and both variants. Two dictionary mutations were previously caught.
- #1752: lazy storage declarations share the Pending/Ready representation schema.
  Strict declaration/runtime-state tests pass; both declaration mutations were caught.
- #1763: React hook handles retain exact payloads and runtime identity. Both the
  React 18 mutable-ref alias and React 19 canonical ref tests pass; the metadata
  reader mutation was caught.


The earlier focused run passed five tests: upstream's nullable-promise
and async declaration regressions plus three retained cases in two downstream files.
A clean application of the combined patch matches every private compiler edit.

Upstream now owns the old Cow declaration, metamorphic partition, Cargo-input,
memory-fixture and pool fixes, the WebAPI promise fix (#1768), and async declaration payloads (#1769). Their old
patches and promise mutation helper are removed. See patches/rust-js/README.md
for the published commits. No private compiler commit is used as the gitlink.

## Broad compiler gates

No interrupted or failing attempt is counted as passing. Against the old
0f641c2 pin, initial runs found a serial metamorphic tail, an order-dependent
Cargo assertion, a slow memory fixture, a lazy-schema architecture issue and
pool accounting/admission defects. Fixes received focused evidence. One run
finished in 546 seconds with an architecture failure later corrected; other
runs reached their enforced 720-second limits without a final suite result.

The last monitored old-pin run passed all fourteen corpus parts and its reported
mutation/format/Clippy stages. It also exposed a child-test startup assumption:
the 100 ms deadline could expire before the fixture wrote its PID, then the test
read the missing file. A bounded delayed-start probe independently reproduced
that condition; it is reported as #1770. The run was stopped with exit 143 at the
user's request to upgrade, before an overall result. Upstream's newer machine
leases, pool backfilling and bounded footprint sampling supersede the old local
validation-loop patches. A later monitored run on b0bc0a3 was also stopped with exit 143 when the user
requested the published async declaration fix. Completed stages are retained as
scoped evidence; no overall passing result is claimed. A full passing gate on
the current pin remains unproven.

## Remaining milestone gates

Finish and render the actual prism scene and every landing section. Exercise
real full-page GPU output, light/dark themes, desktop/mobile framing, interaction,
fallback and lifecycle against upstream. Publish the source, configure a
build/deploy-only Pages workflow with pinned current actions, and verify the
actual public page. Keep the milestone open until all of these are proven.
