import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
import {prepareShaderBindings,ShaderBindings} from '../crates/web/src/bindings.js';
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu');
const {reflectSource}=await import(upstream+'/packages/wgsl/src/runtime/reflect-source.ts');
const source=(wgsl:string)=>({wgsl,reflection:reflectSource(wgsl)});

test('all uniform blocks validate before any GPU allocation, including a later invalid block',()=>{
  const shader=source('struct A { x:f32 }; struct B { count:u32 }; @group(0) @binding(0) var<uniform> a:A; @group(0) @binding(1) var<uniform> b:B;');
  let allocations=0;
  const device={createBuffer(){allocations++;throw Error('No allocation is allowed');}};
  const result=prepareShaderBindings(device,shader,[{name:'a',value:{x:1}},{name:'b',value:{count:-1}}],'preflight');
  expect(result.TAG).toBe('Err');expect(result._0).toContain('integer-range');expect(allocations).toBe(0);
  const duplicate=prepareShaderBindings(device,shader,[{name:'a',value:{x:1}},{name:'a',value:{x:2}}],'duplicate');
  expect(duplicate.TAG).toBe('Err');expect(duplicate._0).toContain('Duplicate');expect(allocations).toBe(0);
  const storage=source('@group(0) @binding(0) var<storage,read> values:array<u32,2>;');
  expect(prepareShaderBindings(device,storage,[{name:'values',value:[1,2]}],'wrong-kind').TAG).toBe('Err');
  expect(allocations).toBe(0);
});

test('rebinding uses reflected slots and retained buffers, rejects missing/duplicate resources and cleans up',()=>{
  const shader=source('struct Params { count:u32 }; @group(0) @binding(3) var<uniform> params:Params; @group(0) @binding(7) var image:texture_2d<f32>; @group(0) @binding(9) var filtering:sampler;');
  let allocated=0,destroyed=0,groups=0;
  const buffer={destroy(){destroyed++;}},uploads:number[]=[];
  const queue={writeBuffer(target:unknown,_offset:number,bytes:ArrayBuffer){expect(target).toBe(buffer);uploads.push(new DataView(bytes).getUint32(0,true));}};
  const device={queue,createBuffer(){allocated++;return buffer;},createBindGroup(descriptor:unknown){groups++;return descriptor;}};
  const pipeline={getBindGroupLayout(index:number){expect(index).toBe(0);return 'native-layout';}};
  const previous=Object.getOwnPropertyDescriptor(globalThis,'GPUBufferUsage');
  Object.assign(globalThis,{GPUBufferUsage:{UNIFORM:64,COPY_DST:8}});
  try{
    const prepared=prepareShaderBindings(device,shader,[{name:'params',value:{count:7}}],'retained');
    expect(prepared.TAG).toBe('Ok');const bindings=prepared._0;
    const imageA={},imageB={},sampler={};
    expect(ShaderBindings.bind(bindings,device,pipeline,[],'missing').TAG).toBe('Err');
    expect(ShaderBindings.bind(bindings,device,pipeline,[{name:'image',resource:imageA},{name:'image',resource:imageA}],'duplicate').TAG).toBe('Err');
    expect(groups).toBe(0);
    for(const image of [imageA,imageB]){
      const result=ShaderBindings.bind(bindings,device,pipeline,[{name:'image',resource:image},{name:'filtering',resource:sampler}],'bound');
      expect(result.TAG).toBe('Ok');
      expect(result._0.entries).toEqual([{binding:3,resource:{buffer}},{binding:7,resource:image},{binding:9,resource:sampler}]);
    }
    expect(allocated).toBe(1);expect(groups).toBe(2);
    expect(ShaderBindings.write(bindings,queue,'params',{count:12}).TAG).toBe('Ok');
    expect(ShaderBindings.write(bindings,queue,'params',{count:-1}).TAG).toBe('Err');
    expect(uploads).toEqual([7,12]);expect(allocated).toBe(1);
    ShaderBindings.destroy(bindings);expect(destroyed).toBe(1);
  }finally{
    if(previous)Object.defineProperty(globalThis,'GPUBufferUsage',previous);else Reflect.deleteProperty(globalThis,'GPUBufferUsage');
  }
});
