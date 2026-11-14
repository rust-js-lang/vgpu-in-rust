import {expect,test} from 'bun:test';
import {mkdirSync,readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {defaultPrismControls} from '../crates/prism/src/controls.js';
import {sceneUniforms,glassUniforms,schlickFresnelF0} from '../crates/prism/src/uniforms.js';
import {cameraView} from '../crates/prism/src/camera.js';
import {lightMeshLayout} from '../crates/prism/src/light_mesh.js';
import {lampForIncidence} from '../crates/prism/src/types.js';
import * as runtimeMath from '../crates/prism/src/runtime.js';
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu');
const source=resolve(upstream,'apps/docs/app/[lang]/(home)/components/prism-background');
const output=resolve(import.meta.dir,'../node_modules/.cache/vgpu-reference-uniforms');
mkdirSync(output,{recursive:true});
await Bun.write(output+'/entry.ts','export * from '+JSON.stringify(source+'/runtime/uniforms.ts')+'; export {incidenceAt,lampAt,wallExtent,lightWallExtent} from '+JSON.stringify(source+'/runtime/state.ts')+';');
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
