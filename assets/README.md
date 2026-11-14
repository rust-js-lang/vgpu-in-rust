# Original shader assets

These 44 WGSL files are unchanged from the vgpu revision recorded in UPSTREAM.md. Their relative module paths are retained, including debug probes used by the original scene tests. The original MIT notice is retained in LICENSE.

The light-theme wall mask under public/hero/prism-light is copied unchanged from the same revision. These assets are used by the Rust GPU renderer and React hero acceptance fixture.

`vgpu/fullscreen.wgsl` retains the exact FULLSCREEN_VERTEX_SOURCE literal from upstream packages/vgpu-api/src/fullscreen-stage.ts. Asset tests compare it with the original, and a browser GPU readback test exercises the Rust fullscreen pipeline with the original copy effect. Both connected prism graphs and the React hero are implemented; complete landing content and deployment remain in progress.

The styles and site directories retain the six landing/application CSS files
from the same upstream revision. Their framework CSS dependencies are pinned
to the original lockfile versions. Application tests compare every retained
stylesheet with the original bytes.
