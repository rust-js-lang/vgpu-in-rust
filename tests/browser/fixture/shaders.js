// Test-only asset inventory and Rust entry points. GPU setup belongs to the test.
import {shaderModule,compilationErrors} from '../../../crates/web/src/shader.js';
import {fullscreenVertex,fullscreenPipeline,encodeFullscreen} from '../../../crates/web/src/fullscreen.js';
const sources=import.meta.glob('../../../assets/prism-background/**/*.wgsl',{eager:true,import:'default'});
Object.assign(window,{shaderFixture:{sources,shaderModule,compilationErrors,fullscreenVertex,fullscreenPipeline,encodeFullscreen}});
