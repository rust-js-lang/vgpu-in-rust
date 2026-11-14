import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
import {tryPack} from '../crates/web/src/packing.js';
const original=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu');
const reflection=await import(original+'/packages/wgsl/src/runtime/reflect-source.ts');
const bundle=await Bun.build({entrypoints:[original+'/packages/vgpu-api/src/set-packing.ts'],target:'bun',plugins:[{name:'original-core-errors',setup(build){build.onResolve({filter:/^@vgpu\/core$/},()=>({path:original+'/packages/core/src/errors.ts'}));}}]});
if(!bundle.success)throw Error(bundle.logs.join('\n'));
const path=resolve('node_modules/.cache/packing-reference.js');
await Bun.write(path,bundle.outputs[0]);
const reference=await import(path);
function layout(source:string){return reflection.reflectSource(source).bindings[0].layout;}
function uniform(declaration:string,type='Values'){return layout(declaration+'\n@group(0) @binding(0) var<uniform> values: '+type+';');}
function compare(shape:any,value:unknown){
  const expected=reference.writeLayoutValue(shape,value);
  const actual=tryPack(shape,value);
  expect(actual.TAG,JSON.stringify(actual)).toBe('Ok');
  expect(actual._0).toBeInstanceOf(ArrayBuffer);
  expect(new Uint8Array(actual._0)).toEqual(new Uint8Array(expected));
  return actual._0 as ArrayBuffer;
}
function invalid(shape:any,value:unknown){
  let expected:any;
  try{reference.writeLayoutValue(shape,value);}catch(error){expected=error;}
  expect(expected).toBeDefined();
  const actual=tryPack(shape,value);
  expect(actual.TAG).toBe('Err');
  for(const key of ['code','message','fix','where','detail'])expect(actual._0[key],key).toEqual(expected[key]);
}

test('reflected uniform packing preserves vector/matrix padding, nested arrays and exact integer bytes',()=>{
  const shape=uniform('struct Inner { v: vec3f, n: u32 }; struct Values { x: f32, direction: vec3f, signed: i32, matrix: mat3x3f, nodes: array<Inner, 2> };');
  for(const scale of [0,0.1,-1,1e-6,1e8])compare(shape,{x:scale,direction:new Float32Array([scale,2,3]),signed:-2147483648,matrix:[1,2,3,4,5,6,7,8,scale],nodes:[{v:[scale,-scale,NaN],n:4294967295},{v:new Float64Array([Infinity,-Infinity,-0]),n:0}]});
  const record=Object.assign(Object.create(null),{x:1,direction:[1,2,3],signed:1,matrix:[1,2,3,4,5,6,7,8,9],nodes:[{v:[1,2,3],n:1},{v:[4,5,6],n:2}]});
  compare(shape,record);
});

test('binary16 rounding agrees with the source for special, boundary, tie and sampled float32 values',()=>{
  const shape=uniform('enable f16; struct Values { x: f16, vector: vec3h, matrix: mat3x2h };');
  const values=[0,-0,NaN,Infinity,-Infinity,65504,65520,2**-24,2**-25,1+2**-11,1+3*2**-11];
  let seed=1234567;
  const storage=new ArrayBuffer(4),bits=new DataView(storage);
  for(let index=0;index<2048;index++){
    seed=(Math.imul(seed,1664525)+1013904223)>>>0;
    bits.setUint32(0,seed,true);
    values.push(bits.getFloat32(0,true));
  }
  for(const value of values)compare(shape,{x:value,vector:[value,-value,value],matrix:[value,0,value,1,value,-1]});
});

test('structured validation preserves source error codes, paths, shapes and integer ranges',()=>{
  const shape=uniform('struct Values { vector: vec3f, matrix: mat2x2f, count: u32 };');
  const valid={vector:[1,2,3],matrix:[1,2,3,4],count:1};
  for(const value of [null,undefined,[],new Date(),new Map(),{}, {...valid,extra:1},{...valid,vector:[1,2]}, {...valid,vector:[1,2,3,4]}, {...valid,vector:new DataView(new ArrayBuffer(12))}, {...valid,vector:new BigInt64Array([1n,2n,3n])},{...valid,matrix:[1,2,3]}, ...[-1,0.5,NaN,Infinity,4294967296].map(count=>({...valid,count}))])invalid(shape,value);
  const hole=[1,,3];invalid(shape,{...valid,vector:hole});
  const signed=uniform('struct Values { count: i32 };');
  for(const count of [-2147483649,2147483648,0.1,Infinity])invalid(signed,{count});
  const nested=uniform('struct Node { x: f32 }; struct Values { nodes: array<Node,2> };');
  invalid(nested,{nodes:[{x:1},{}]});
  invalid(nested,{nodes:[{x:1}]});
  const tooSmall={...shape,size:4};invalid(tooSmall,valid);
  const badOffset={...shape,members:shape.members.map((member:any,index:number)=>index===0?{...member,offset:-1}:member)};invalid(badOffset,valid);
});

test('runtime array extents and unsupported layouts follow the source validation decisions',()=>{
  const shape=layout('@group(0) @binding(0) var<storage,read_write> values: array<u32>;');
  invalid(shape,[1,2]);
  compare({...shape,size:8},new Uint32Array([1,4294967295]));
  invalid({...shape,size:12},[1,2]);
  invalid({...shape,size:8,stride:Infinity},[1,2]);
  invalid({...shape,size:8,type:{kind:'texture'}},[1,2]);
});

test('write-time getters retain source coercion, nullish vector defaults and access order',()=>{
  const shape=uniform('struct Values { vector: vec3f };');
  for(const after of [undefined,null,[4,5,6],[null,undefined,'7']]){
    const make=()=>{const log:string[]=[];let reads=0;const value=Object.defineProperty({},'vector',{enumerable:true,get(){log.push('read');return ++reads===1?[1,2,3]:after;}});return {value,log};};
    const a=make(),b=make();
    const expected=reference.writeLayoutValue(shape,a.value),actual=tryPack(shape,b.value);
    expect(actual.TAG).toBe('Ok');
    expect(new Uint8Array(actual._0)).toEqual(new Uint8Array(expected));
    expect(b.log).toEqual(a.log);
  }
  const scalar=uniform('struct Values { x: f32 };');
  const make=()=>{let count=0;return Object.defineProperty({},'x',{enumerable:true,get(){return ++count===1?1:3n;}});};
  expect(()=>reference.writeLayoutValue(scalar,make())).toThrow(TypeError);
  expect(()=>tryPack(scalar,make())).toThrow(TypeError);
});


test('the actual Scene and Glass WGSL structs pack Rust uniform values byte-for-byte like upstream',async()=>{
  const {defaultPrismControls}=await import('../crates/prism/src/controls.js');
  const {sceneUniforms,glassUniforms}=await import('../crates/prism/src/uniforms.js');
  const {cameraView}=await import('../crates/prism/src/camera.js');
  const {lampForIncidence}=await import('../crates/prism/src/types.js');
  const {lightMeshLayout}=await import('../crates/prism/src/light_mesh.js');
  async function shaderStruct(path:string,name:string){
    const text=await Bun.file(resolve('assets/prism-background',path)).text();
    const definition=text.match(/export struct \w+ \{[\s\S]*?\n\}/g)?.find(definition=>definition.startsWith('export struct '+name+' '));
    expect(definition).toBeDefined();
    return uniform(definition!.replace('export ',''),name);
  }
  const scene=await shaderStruct('scene/scene.wgsl','Scene');
  const glass=await shaderStruct('pipelines/shared/glass/glass-common.wgsl','Glass');
  for(const mode of ['dark','light'])for(const aspect of [0.35,1,16/9])for(const reveal of [0,0.4,1]){
    const controls=defaultPrismControls();
    const view=cameraView(aspect,0.4,-0.7,1.25,48);
    compare(scene,sceneUniforms(view,[2.8,1.6],lampForIncidence(60,0.025,0.5),controls,lightMeshLayout(64,12),reveal));
    compare(glass,glassUniforms(view,controls,mode,true));
  }
});
