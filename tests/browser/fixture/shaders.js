// Test-only asset inventory and Rust entry points. GPU setup belongs to the test.
import {shaderModule,compilationErrors} from '../../../crates/web/src/shader.js';
import {fullscreenVertex,fullscreenPipeline,encodeFullscreen} from '../../../crates/web/src/fullscreen.js';
const sources=import.meta.glob('../../../assets/prism-background/**/*.wgsl',{eager:true,import:'default'});
Object.assign(window,{shaderFixture:{sources,shaderModule,compilationErrors,fullscreenVertex,fullscreenPipeline,encodeFullscreen}});

import {lightPipeline,encodeLight} from '../../../crates/web/src/spectral_pass.js';
import {buildLightMesh} from '../../../crates/web/src/light_mesh.js';
import {lampForIncidence,STYLIZED_DISPERSION} from '../../../crates/prism/src/types.js';
Object.assign(window.shaderFixture,{lightPipeline,encodeLight,buildLightMesh,lampForIncidence,STYLIZED_DISPERSION});
