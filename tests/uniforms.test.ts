import {expect,test} from 'bun:test';
import {mkdirSync,readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {defaultPrismControls} from '../crates/prism/src/controls.js';
import {sceneUniforms,glassUniforms,schlickFresnelF0} from '../crates/prism/src/uniforms.js';
import {normalizeControls} from '../crates/prism/src/normalize_controls.js';
import {buildLightMesh} from '../crates/web/src/light_mesh.js';
import {darkWallClear,srgbToLinear} from '../crates/prism/src/color.js';
import {RuntimeState} from '../crates/prism/src/runtime_state.js';
import {cameraView} from '../crates/prism/src/camera.js';
import {lightMeshLayout} from '../crates/prism/src/light_mesh.js';
import {lampForIncidence} from '../crates/prism/src/types.js';
import * as runtimeMath from '../crates/prism/src/runtime.js';
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu');
const source=resolve(upstream,'apps/docs/app/[lang]/(home)/components/prism-background');
const output=resolve(import.meta.dir,'../node_modules/.cache/vgpu-reference-uniforms');
mkdirSync(output,{recursive:true});
await Bun.write(output+'/entry.ts','export * from '+JSON.stringify(source+'/runtime/uniforms.ts')+'; export * from '+JSON.stringify(source+'/runtime/state.ts')+'; export {normalizeControls} from '+JSON.stringify(source+'/runtime/normalize-controls.ts')+'; export {cameraView as referenceCameraView} from '+JSON.stringify(source+'/scene/camera.ts')+'; export {darkWallClear,srgbToLinear} from '+JSON.stringify(source+'/pipelines/dark/passes/wall/clear.ts')+';');
const built=await Bun.build({entrypoints:[output+'/entry.ts'],outdir:output+'/out',target:'bun',format:'esm',
  plugins:[{name:'pure-uniform-reference',setup(build){
    build.onResolve({filter:/^vgpu\/scene$/},()=>({path:'scene',namespace:'original-scene'}));
    build.onLoad({filter:/.*/,namespace:'original-scene'},()=>({loader:'ts',contents:
      'export {group} from '+JSON.stringify(resolve(upstream,'packages/vgpu-api/src/scene/nodes.ts'))+';\n'+
      'export {perspective,viewMatrices} from '+JSON.stringify(resolve(upstream,'packages/vgpu-api/src/scene/camera-state.ts'))+';'}));
    build.onResolve({filter:/^@vgpu\/core$/},()=>({path:resolve(upstream,'packages/core/src/errors.ts')}));
    build.onLoad({filter:/\/(scene\/prism-mesh|environment\/texture)\.ts$/},({path})=>{
      // Only pure exports are reachable from uniforms. Remove imports used by
      // unreachable GPU upload/bake functions; all reference bodies and
      // constants remain verbatim. No renderer behavior is substituted.
      let text=readFileSync(path,'utf8');
      text=text.replace(/^import \{ (?:geometry|effect, frame, sampler, target) \} from "vgpu";$/m,'');
      text=text.replace(/^import \w+ from "\.\/environment-(?:bake|blur)\.wgsl";$/gm,'');
      return {loader:'ts',contents:text};
    });
  }}]});
if(!built.success)throw new AggregateError(built.logs,'Cannot compile upstream uniform reference');
const reference=await import(built.outputs.find(file=>file.path.endsWith('/entry.js'))!.path);
const originalTypes=await import(source+'/types.ts');

function plain(value:unknown):unknown {
  if(ArrayBuffer.isView(value))return Array.from(value as unknown as ArrayLike<number>);
  if(Array.isArray(value))return value.map(plain);
  if(value&&typeof value==='object')return Object.fromEntries(Object.entries(value).map(([key,value])=>[key,plain(value)]));
  return value;
}

test('all scene control defaults retain the upstream runtime schema and values',()=>{
  expect(defaultPrismControls()).toEqual(originalTypes.DEFAULT_PRISM_CONTROLS);
});

test('scene uniforms preserve source values, alternate mesh tiers, reveal clamps and matrix identity',()=>{
  for(const aspect of [0.35,1,16/9])for(const [samples,slices]of [[128,24],[64,12],[8,3]])
  for(const color of ['#000000','#aB80fF','123abc','#fff','abcdefg',' #000000','#ffffff\n','ééé'])
  for(const reveal of [-1,0,0.23,1,3,NaN]){
    const controls=defaultPrismControls(); controls.wallColor=color;controls.view=reveal===0?'caustic':'glass';
    controls.lightFade.beamOpacity=0.63;
    const view=cameraView(aspect,0.4,-0.7,1.25,48);
    const layout=lightMeshLayout(samples,slices);
    const runtime={aspect,cameraDistance:1.25,controls,view,lampArc:0.5,lampTarget:0.5,
      framing:{scale:0.8,offset:[0.1,-0.2]},lightMeshLayout:layout};
    const expected=reference.sceneUniforms(runtime,reveal);
    const actual=sceneUniforms(view,expected.wallHalfExtent,lampForIncidence(60,controls.beamWidth,0.5),controls,layout,reveal);
    expect(plain(actual)).toEqual(plain(expected));
    expect(actual.viewProjection).toBe(view.viewProjection);
  }
});

test('both glass themes retain optical uniforms, debug gating and shared immutable arrays',()=>{
  for(const mode of ['dark','light'])for(const enabled of [false,true])for(const debug of [false,true])
  for(const ior of [1,1.0001,1.3,1.645,2.5,NaN,Infinity]){
    const controls=defaultPrismControls();controls.environmentDebug=debug;
    controls.glass.transmission[mode].ior=ior;
    const view=cameraView(16/9,-0.3,0.7,1.25,48);
    const runtime={view,controls,debugEnvironmentEnabled:enabled};
    const actual=glassUniforms(view,controls,mode,enabled);
    expect(plain(actual)).toEqual(plain(reference.glassUniforms(runtime,mode)));
    const second=glassUniforms(view,controls,mode,enabled);
    expect(actual.viewProjection).toBe(view.viewProjection);
    expect(actual.environmentRotation).toBe(second.environmentRotation);
    expect(actual.prismPlanes).toBe(second.prismPlanes);
    expect(schlickFresnelF0(ior)).toBe(reference.schlickFresnelF0(ior));
  }
});


test('runtime lamp endpoints and portrait wall overscan match upstream without changing optics',()=>{
  for(const endpoints of [{top:-35,bottom:75},{top:-85,bottom:85},{top:75,bottom:-35}])
  for(const position of [-1,0,0.1,0.5,0.8,1,2]){
    expect(runtimeMath.incidenceAt(position,endpoints)).toBe(reference.incidenceAt(position,endpoints));
    expect(runtimeMath.lampAt(position,0.025,0.5,endpoints)).toEqual(reference.lampAt(position,0.025,0.5,endpoints));
  }
  for(const aspect of [1e-4,0.2,0.35,1,16/9,4])for(const distance of [0.8,1.25,3])
  for(const framing of [{scale:1,offset:[0,0]},{scale:0.7,offset:[0.2,-0.3]}]){
    for(const method of ['wallExtent','lightWallExtent']){
      const actual=runtimeMath[method](aspect,distance,48,framing);
      const expected=reference[method](aspect,distance,48,framing);
      expect(actual[0]).toBeCloseTo(expected[0],12);
      expect(actual[1]).toBeCloseTo(expected[1],12);
    }
  }
});

test('typed normalization matches upstream for every numeric field and preserves input objects',()=>{
  const paths:string[][]=[];
  function fields(object:any,path:string[]=[]){for(const [key,value]of Object.entries(object)){
    if(typeof value==='number')paths.push([...path,key]);else if(value&&typeof value==='object')fields(value,[...path,key]);
  }}
  fields(originalTypes.DEFAULT_PRISM_CONTROLS);
  for(const path of paths)for(const value of [-Infinity,-100,0,0.25,2,100,Infinity,NaN]){
    const input=structuredClone(originalTypes.DEFAULT_PRISM_CONTROLS);let item=input;
    for(const key of path.slice(0,-1))item=item[key];item[path.at(-1)!]=value;
    const before=structuredClone(input),expected=reference.normalizeControls(input),actual=normalizeControls(input);
    expect(actual,path.join('.')).toEqual(expected);expect(input).toEqual(before);
    expect(actual).not.toBe(input);expect(actual.glass.transmission.dark.absorption).not.toBe(input.glass.transmission.dark.absorption);
  }
  for(const dispersion of ['stylized','crown','flint']){
    const input=structuredClone(originalTypes.DEFAULT_PRISM_CONTROLS);input.dispersion=dispersion;delete input.spectralDispersion;
    expect(normalizeControls(input)).toEqual(reference.normalizeControls(input));
  }
});

test('retained state follows upstream resizing, fit, orbit, optics and invalidation decisions',()=>{
  for(const size of [[1280,720],[390,844]]){
    const layout=lightMeshLayout(8,3),actual=RuntimeState.new(size,layout);
    let uploads=0;
    const expected:any={outputSize:size,controls:reference.normalizeControls(structuredClone(originalTypes.DEFAULT_PRISM_CONTROLS)),lampArc:originalTypes.PRISM_DEFAULT_ARC,lampTarget:0.5,orbit:[0,0],aspect:size[0]/Math.max(1,size[1]),cameraDistance:originalTypes.CAMERA_DISTANCE,framing:{scale:1,offset:[0,0]},view:reference.referenceCameraView(size[0]/Math.max(1,size[1]),0,0,originalTypes.CAMERA_DISTANCE,48),lightMeshLayout:layout,
      lightVertices:new Float32Array(layout.vertexCount*3),lightVertexScratch:[],lightBuffer:{write(){uploads++;}}};
    function compare(){
      for(const key of ['outputSize','controls','lampArc','lampTarget','orbit','aspect','cameraDistance','framingViewport','framing'])expect(actual[key]).toEqual(expected[key]);
      for(const key of ['position','forward','right','up'])for(let axis=0;axis<3;axis++)expect(actual.view[key][axis]).toBeCloseTo(expected.view[key][axis],14);
      expect(plain(actual.view.viewProjection)).toEqual(plain(expected.view.viewProjection));
      if(uploads>0)expect(Array.from(buildLightMesh(RuntimeState.lightMeshOptions(actual)).vertices)).toEqual(Array.from(expected.lightVertices));
    }
    compare();const matrix=actual.view.viewProjection;
    expect(RuntimeState.resize(actual,size)).toBe(false);reference.resizeRuntime(expected,size);expect(actual.view.viewProjection).toBe(matrix);compare();
    for(const viewport of [{left:0.4,top:0.1,right:0.95,bottom:0.9},{left:0.1,top:0.05,right:0.9,bottom:0.45},undefined]){
      const before=uploads;const changed=RuntimeState.setFramingViewport(actual,viewport);reference.setRuntimeFramingViewport(expected,viewport);expect(changed).toBe(uploads>before);compare();
      if(viewport){
        const same={...viewport,left:viewport.left+1e-6};const before=uploads;
        expect(RuntimeState.setFramingViewport(actual,same)).toBe(false);reference.setRuntimeFramingViewport(expected,same);expect(uploads).toBe(before);compare();
      }
      for(const orbit of [[-0.7,0.6],[2,-2],[0,0]]){RuntimeState.setOrbit(actual,...orbit);reference.setRuntimeOrbit(expected,...orbit);compare();}
      for(const aim of [[0.5,0.5],[-1,2],[0.2,0.7]]){
        const before=uploads;expect(RuntimeState.setLampAim(actual,...aim)).toBe((reference.setRuntimeLampAim(expected,...aim),uploads>before));compare();
      }
      const controls=structuredClone(actual.controls);controls.cameraFov=55;controls.beamWidth=0.08;
      const beforeControls=uploads;expect(RuntimeState.setControls(actual,controls)).toBe((reference.setRuntimeControls(expected,controls),uploads>beforeControls));compare();
      const beforeMatrix=actual.view.viewProjection;const presentation=structuredClone(actual.controls);presentation.postprocess.bloomRadius=2.5;
      expect(RuntimeState.setControls(actual,presentation)).toBe(false);reference.setRuntimeControls(expected,presentation);compare();expect(actual.view.viewProjection).toBe(beforeMatrix);
    }
  }
});

test('dark wall clear preserves strict color parsing and the original sRGB transfer',()=>{
  for(const color of ['#000000','#305070','aB80fF','#ffffff','#fff','ééé','#ffffff\n',' #305070','3050700'])for(const view of ['glass','wall','back','caustic'])
    expect(darkWallClear(color,view)).toEqual(reference.darkWallClear(color,view));
  for(const value of [-1,0,0.04045,0.040450001,0.5,1,2,NaN,Infinity])expect(srgbToLinear(value)).toBe(reference.srgbToLinear(value));
});
