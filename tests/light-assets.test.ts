import {expect,test} from 'bun:test';
import {mkdirSync,readFileSync} from 'node:fs';import {resolve} from 'node:path';
import * as assets from '../crates/prism/src/light_assets.js';
import * as math from '../crates/prism/src/light_asset_math.js';
import {buildShadowMesh} from '../crates/prism/src/shadow_mesh.js';
const base=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu','apps/docs/app/[lang]/(home)/components/prism-background');
const out=resolve(import.meta.dir,'../node_modules/.cache/vgpu-reference-light-assets');mkdirSync(out,{recursive:true});
await Bun.write(out+'/entry.ts',[
  'export * from '+JSON.stringify(base+'/pipelines/light/assets/generate.ts')+';',
  'export * from '+JSON.stringify(base+'/pipelines/light/assets/generate-wall.ts')+';',
  'export * from '+JSON.stringify(base+'/pipelines/light/assets/generate-caustic.ts')+';',
  'export * from '+JSON.stringify(base+'/pipelines/light/assets/mips.ts')+';',
  'export * from '+JSON.stringify(base+'/pipelines/light/assets/math.ts')+';',
  'export {buildShadowMesh} from '+JSON.stringify(base+'/pipelines/light/passes/shadow/mesh.ts')+';',
].join('\n'));
const built=await Bun.build({entrypoints:[out+'/entry.ts'],outdir:out+'/out',target:'bun',format:'esm',plugins:[{
  name:'pure-shadow-reference',setup(build){build.onLoad({filter:/\/shadow\/mesh\.ts$/},({path})=>({loader:'ts',
    // Only the pure builder is exported. Retain its exact body; the unreachable
    // GPU uploader is tree-shaken after removing its unused runtime import.
    contents:readFileSync(path,'utf8').replace(/^import \{ geometry \} from "vgpu";$/m,''),
  }));}
}]});if(!built.success)throw new AggregateError(built.logs,'Cannot build original light-asset reference');
const ref=await import(built.outputs.find(output=>output.path.endsWith('/entry.js'))!.path);
function value(result:any){if(result.TAG==='Err')throw Error(result._0);return result._0;}

test('CPU recovery textures and every mip match original bytes including single axes and odd dimensions',()=>{
  for(const [actual,original]of [[assets.generateWallMaterial,ref.generateWallMaterial],[assets.generateWallLighting,ref.generateWallLighting],[assets.generateCausticProfile,ref.generateCausticProfile]])
  for(const size of [[1,1],[1,9],[9,1],[7,11],[32,24],[96,64]]){
    const image=actual(size),expected=original(size);expect(image.width).toBe(expected.width);expect(image.height).toBe(expected.height);expect(image.pixels).toEqual(Array.from(expected.pixels));
    const levels=assets.generateMipChain(image),mips=ref.generateMipChain(expected);expect(levels.length).toBe(mips.length);
    for(let level=0;level<levels.length;level++){expect([levels[level].width,levels[level].height]).toEqual([mips[level].width,mips[level].height]);expect(levels[level].pixels).toEqual(Array.from(mips[level].pixels));}
  }
});
test('mask resampling preserves bilinear coordinates, edge fades and untouched channels',()=>{
  for(const [width,height]of [[1,1],[7,5],[32,24]])for(const [sourceWidth,sourceHeight]of [[1,1],[3,7],[15,11]]){
    const source=Uint8Array.from({length:sourceWidth*sourceHeight*4},(_,index)=>(index*97)%256);
    const original=ref.generateWallLighting([width,height]),actual=assets.generateWallLighting([width,height]);
    ref.applyGlobalLightMask(original,source,sourceWidth,sourceHeight);assets.applyGlobalLightMask(actual,Array.from(source),sourceWidth,sourceHeight);
    expect(actual.pixels).toEqual(Array.from(original.pixels));
    for(const inset of [-1,0,1,3,40,NaN])expect(assets.globalLightMaskEdgeMax(actual.pixels,width,height,inset)).toBe(ref.globalLightMaskEdgeMax(original.pixels,width,height,inset));
  }
});
test('negative-coordinate procedural noise and octave weights retain source values',()=>{
  for(const x of [-35.3,-1,0,0.25,17.8])for(const y of [-9.2,0,0.7,56.1]){
    expect(math.noise(x,y)).toBe(ref.noise(x,y));for(const octaves of [1,2,4,5])expect(math.fbm(x,y,octaves)).toBe(ref.fbm(x,y,octaves));
  }
});
test('shadow hull, all Float32 vertices and aligned uint32 topology match the original sweep',()=>{
  const triangles=[{a:[0,1],b:[-1,0],c:[1,0]},{a:[0.2,0.7],b:[0.5,-0.3],c:[-0.6,-0.2]}];
  for(const triangle of triangles)for(const projection of [[0,0],[0.5,-0.7],[-0.5,0.7],[0.7,0],[5,-6]]){
    const options={projection,nearPenumbra:0.00855,farPenumbra:0.057,midRing:0.48,midCoverage:0.32};
    const actual=value(buildShadowMesh(triangle,options)),expected=ref.buildShadowMesh(triangle,options);
    expect(actual.vertices).toEqual(Array.from(expected.vertices));expect(actual.indices).toEqual(Array.from(expected.indices));expect(actual.hull).toEqual(expected.hull);expect(actual.hullTravel).toEqual(expected.hullTravel);
  }
  const triangle={a:[0,0],b:[0,0],c:[0,0]},options={projection:[0,0],nearPenumbra:0,farPenumbra:0,midRing:0,midCoverage:0};
  expect(buildShadowMesh(triangle,options)).toEqual({TAG:'Err',_0:'A cast-shadow hull needs three points.'});
});

test('the authored wall mask remains byte-identical to upstream',async()=>{
  expect(await Bun.file(resolve(import.meta.dir,'../assets/prism-light/wall-global-light-mask.webp')).arrayBuffer()).toEqual(await Bun.file(resolve(base,'../../../../../public/hero/prism-light/wall-global-light-mask.webp')).arrayBuffer());
});
