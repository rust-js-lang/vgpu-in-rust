import {expect,test} from 'bun:test';
import {createUniformBuffer,writeUniformBuffer} from '../crates/web/src/buffers.js';
import {resolve} from 'node:path';
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu');
const {reflectSource}=await import(upstream+'/packages/wgsl/src/runtime/reflect-source.ts');

test('invalid uniforms allocate nothing and rejected updates leave the retained GPU buffer untouched',()=>{
  const layout=reflectSource('struct Values { count: u32 }; @group(0) @binding(0) var<uniform> params: Values;').bindings[0].layout;
  let allocations=0;const uploads:ArrayBuffer[]=[];
  const buffer={};
  const queue={writeBuffer(target:unknown,_offset:number,bytes:ArrayBuffer){expect(target).toBe(buffer);uploads.push(bytes.slice(0));}};
  const device={queue,createBuffer(){allocations++;return buffer;}};
  // This must reject before touching the browser API, including its globals.
  expect(createUniformBuffer(device,layout,{count:-1},'invalid').TAG).toBe('Err');
  expect(allocations).toBe(0);expect(uploads.length).toBe(0);
  const old=Object.getOwnPropertyDescriptor(globalThis,'GPUBufferUsage');
  try{
    Object.assign(globalThis,{GPUBufferUsage:{UNIFORM:64,COPY_DST:8}});
    const result=createUniformBuffer(device,layout,{count:7},'valid');
    expect(result.TAG).toBe('Ok');
    const resource=result._0;
    expect(allocations).toBe(1);expect(resource.buffer).toBe(buffer);
    expect(new DataView(uploads[0]).getUint32(0,true)).toBe(7);
    expect(writeUniformBuffer(resource,queue,{count:0.5}).TAG).toBe('Err');
    expect(allocations).toBe(1);expect(uploads.length).toBe(1);
    expect(writeUniformBuffer(resource,queue,{count:11}).TAG).toBe('Ok');
    expect(allocations).toBe(1);expect(uploads.length).toBe(2);
    expect(new DataView(uploads[1]).getUint32(0,true)).toBe(11);
  }finally{
    if(old)Object.defineProperty(globalThis,'GPUBufferUsage',old);else Reflect.deleteProperty(globalThis,'GPUBufferUsage');
  }
});
