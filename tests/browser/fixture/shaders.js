// Test-only asset inventory and Rust entry points. GPU setup belongs to the test.
import {shaderModule,compilationErrors} from '../../../crates/web/src/shader.js';
import {fullscreenVertex,fullscreenPipeline,encodeFullscreen} from '../../../crates/web/src/fullscreen.js';
const sources=import.meta.glob('../../../assets/prism-background/**/*.wgsl',{eager:true,import:'default'});
Object.assign(window,{shaderFixture:{sources,shaderModule,compilationErrors,fullscreenVertex,fullscreenPipeline,encodeFullscreen}});

import {lightPipeline,encodeLight} from '../../../crates/web/src/spectral_pass.js';
import {buildLightMesh} from '../../../crates/web/src/light_mesh.js';
import {lampForIncidence,STYLIZED_DISPERSION} from '../../../crates/prism/src/types.js';
Object.assign(window.shaderFixture,{lightPipeline,encodeLight,buildLightMesh,lampForIncidence,STYLIZED_DISPERSION});

import {tryPack} from '../../../crates/web/src/packing.js';
Object.assign(window.shaderFixture,{tryPack});

import {sceneUniforms,glassUniforms} from '../../../crates/prism/src/uniforms.js';
import {defaultPrismControls} from '../../../crates/prism/src/controls.js';
import {lightMeshLayout} from '../../../crates/prism/src/light_mesh.js';
Object.assign(window.shaderFixture,{sceneUniforms,glassUniforms,defaultPrismControls,lightMeshLayout});

import {createUniformBuffer,writeUniformBuffer,createVertexBuffer} from '../../../crates/web/src/buffers.js';
Object.assign(window.shaderFixture,{createUniformBuffer,writeUniformBuffer,createVertexBuffer});

import {glassPipeline,encodeGlass} from '../../../crates/web/src/glass_pass.js';
Object.assign(window.shaderFixture,{glassPipeline,encodeGlass});

import {createEnvironmentTexture,createEnvironmentSampler,EnvironmentTexture} from '../../../crates/web/src/environment.js';
import {createRenderTarget,RenderTarget} from '../../../crates/web/src/targets.js';
import {prepareShaderBindings,ShaderBindings} from '../../../crates/web/src/bindings.js';
Object.assign(window.shaderFixture,{createEnvironmentTexture,createEnvironmentSampler,EnvironmentTexture,
  createRenderTarget,RenderTarget,prepareShaderBindings,ShaderBindings,
  referenceEnvironment:()=>import('./reference-environment.js')});

import {createPrismGeometry,PrismBuffers} from '../../../crates/web/src/geometry.js';
import {cameraView} from '../../../crates/prism/src/camera.js';
Object.assign(window.shaderFixture,{createPrismGeometry,PrismBuffers,cameraView});

import {wallExtent} from '../../../crates/prism/src/runtime.js';
Object.assign(window.shaderFixture,{wallExtent});
