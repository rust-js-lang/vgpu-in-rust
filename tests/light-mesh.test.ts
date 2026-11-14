import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
import * as mesh from '../crates/prism/src/light_mesh.js';
import * as scene from '../crates/prism/src/types.js';
import {buildLightMesh as uploadMesh,lightVertexMetadata as uploadMetadata} from '../crates/web/src/light_mesh.js';
const original=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu','apps/docs/app/[lang]/(home)/components/prism-background');
const reference=await import(original+'/scene/light-mesh.ts');
const sourceTypes=await import(original+'/types.ts');

function close(actual:unknown,expected:unknown){
  if(typeof expected==='number'){
    if(Number.isFinite(expected))expect(actual as number).toBeCloseTo(expected,12);
    else expect(Object.is(actual,expected)).toBe(true);
  }else if(Array.isArray(expected)){
    expect((actual as unknown[]).length).toBe(expected.length);
    expected.forEach((value,index)=>close((actual as unknown[])[index],value));
  }else if(expected&&typeof expected==='object'){
    expect(Object.keys(actual as object).sort()).toEqual(Object.keys(expected).sort());
    for(const [key,value]of Object.entries(expected))close((actual as Record<string,unknown>)[key],value);
  }else expect(actual).toEqual(expected);
}

test('spectral layout and every static vertex attribute match original high and low tiers',()=>{
  expect(mesh.HIGH_LIGHT_MESH_LAYOUT).toEqual(reference.HIGH_LIGHT_MESH_LAYOUT);
  for(const [samples,slices]of [[2,1],[8,3],[16,4],[64,12],[128,24]]){
    const layout=mesh.lightMeshLayout(samples,slices);
    expect(layout).toEqual(reference.lightMeshLayout(samples,slices));
    expect(mesh.lightVertexCount(samples,slices)).toBe(reference.lightVertexCount(samples,slices));
    for(let index=0;index<layout.vertexCount;index++){
      expect(mesh.lightVertexMetadata(index,samples,slices)).toEqual(reference.lightVertexMetadata(index,samples,slices));
    }
  }
  for(const [samples,slices]of [[0,0],[1,0],[0,3]])expect(mesh.lightMeshLayout(samples,slices)).toEqual(reference.lightMeshLayout(samples,slices));
});

test('finite beam overlap, wall clipping and physical spectral boundaries match',()=>{
  const triangle=sourceTypes.PRISM_TRIANGLE;
  for(const angle of [-30,0,35,60,85])for(const width of [0.01,0.025,0.15])for(const target of [0.15,0.5,0.85]){
    const light=sourceTypes.lampForIncidence(angle,width,target);
    expect(mesh.beamIntersectsTriangle(triangle,light)).toBe(reference.beamIntersectsTriangle(triangle,light));
    for(const profile of [-2,-1,-0.3,0,0.7,1,2])close(mesh.beamProfileOrigin(light,profile),reference.beamProfileOrigin(light,profile));
    close(mesh.beamBoundaryOrigins(light),reference.beamBoundaryOrigins(light));
    for(const wavelength of [400,475,550,625,700])close(mesh.traceSpectralBand(triangle,light,sourceTypes.PRISM_DISPERSION_PRESETS.stylized,wavelength),reference.traceSpectralBand(triangle,light,sourceTypes.PRISM_DISPERSION_PRESETS.stylized,wavelength));
  }
  for(const origin of [[0,0],[3,0],[-3,1],[0,3],[2,1]])for(const direction of [[1,0],[0,-1],[-1,0.2],[0,0],[1e-10,1]]){
    close(mesh.rayToWallBoundary(origin,direction,[2,1]),reference.rayToWallBoundary(origin,direction,[2,1]));
    close(mesh.lineThroughWall(origin,direction,[2,1]),reference.lineThroughWall(origin,direction,[2,1]));
  }
});

test('packed beam generation preserves every Float32 vertex, energy statistics and buffer reuse',()=>{
  const scratch:number[]=[123,456];
  const cases=[
    {samples:128,beamSlices:24,angle:60,width:0.025,target:0.5,dispersion:scene.STYLIZED_DISPERSION},
    ...[-15,0,35,60,85].flatMap(angle=>[0.01,0.15].map(width=>({samples:16,beamSlices:4,angle,width,target:0.2,dispersion:scene.STYLIZED_DISPERSION}))),
    {samples:32,beamSlices:6,angle:60,width:0.025,target:0.8,dispersion:scene.CROWN_DISPERSION},
    {samples:32,beamSlices:6,angle:60,width:0.025,target:0.5,dispersion:scene.FLINT_DISPERSION},
  ];
  for(const item of cases){
    const light=sourceTypes.lampForIncidence(item.angle,item.width,item.target);
    const options={light,dispersion:item.dispersion,wallHalfExtent:[2.8,1.6],samples:item.samples,beamSlices:item.beamSlices};
    const expected=reference.buildLightMesh(options);
    const stats=mesh.buildLightMesh(options,scratch);
    expect(scratch.length).toBe(expected.vertices.length);
    expect(Float32Array.from(scratch),JSON.stringify(item)).toEqual(expected.vertices);
    close(stats,expected.stats);
  }
  const miss={light:{center:[0,4],direction:[1,0],beamHalfWidth:0.01},dispersion:scene.STYLIZED_DISPERSION,wallHalfExtent:[2,1],samples:8,beamSlices:3};
  const expected=reference.buildLightMesh(miss);
  close(mesh.buildLightMesh(miss,scratch),expected.stats);
  expect(Float32Array.from(scratch)).toEqual(expected.vertices);
});


test('GPU upload uses the original retained Float32 target, overwrites scratch and preserves native errors',()=>{
  const options={light:sourceTypes.lampForIncidence(60,0.025,0.5),dispersion:scene.STYLIZED_DISPERSION,wallHalfExtent:[2.8,1.6],samples:32,beamSlices:6};
  const expected=reference.buildLightMesh(options);
  const target=new Float32Array(expected.vertices.length);
  const scratch:number[]=[NaN,NaN];
  const actual=uploadMesh(options,target,scratch);
  expect(actual.vertices).toBe(target);
  expect(actual.vertexCount).toBe(expected.vertexCount);
  expect(target).toEqual(expected.vertices);
  close(actual.stats,expected.stats);
  expect(scratch.length).toBe(target.length);
  const allocated=uploadMesh(options);
  expect(allocated.vertices).toBeInstanceOf(Float32Array);
  expect(allocated.vertices).not.toBe(target);
  expect(allocated.vertices).toEqual(target);
  const next={...options,light:sourceTypes.lampForIncidence(35,0.1,0.2)};
  const revised=uploadMesh(next,target,scratch);
  expect(revised.vertices).toBe(target);
  expect(target).toEqual(reference.buildLightMesh(next).vertices);
  expect(()=>uploadMesh(options,new Float32Array(3),scratch)).toThrow(RangeError);
  expect(()=>uploadMesh(options,new Float32Array(3),scratch)).toThrow('Light mesh target has 3 floats; expected '+target.length+'.');
  expect(()=>reference.buildLightMesh(options,new Float32Array(3),scratch)).toThrow(RangeError);
});


test('public metadata decoder retains default arguments and the original native RangeError for invalid numbers',()=>{
  for(const vertex of [0,1,143,1000])expect(uploadMetadata(vertex)).toEqual(reference.lightVertexMetadata(vertex));
  for(const vertex of [NaN,Infinity,-Infinity,-1,-0.5,1.5,reference.HIGH_LIGHT_MESH_LAYOUT.vertexCount]){
    let original:Error|undefined;
    try{reference.lightVertexMetadata(vertex);}catch(error){original=error as Error;}
    expect(original).toBeInstanceOf(RangeError);
    expect(()=>uploadMetadata(vertex)).toThrow(RangeError);
    expect(()=>uploadMetadata(vertex)).toThrow(original!.message);
  }
});
