import {expect,test} from '@playwright/test';

test('all original prism WGSL modules compile on a real WebGPU device through Rust',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    if(!navigator.gpu)throw Error('WebGPU is required for this acceptance test');
    const adapter=await navigator.gpu.requestAdapter();
    if(!adapter)throw Error('A real WebGPU adapter is required; a fallback screenshot cannot pass');
    const device=await adapter.requestDevice();
    try{
      const fixture=Reflect.get(window,'shaderFixture');
      const sources=Object.entries(fixture.sources) as [string,{version:number,wgsl:string,reflection:object}][];
      const checks=await Promise.all(sources.map(async([name,source])=>{
        const module=fixture.shaderModule(device,source,name);
        return {name,errors:await fixture.compilationErrors(module),version:source.version,hasReflection:Boolean(source.reflection)};
      }));
      return {checks};
    }finally{device.destroy();}
  });
  expect(report.checks).toHaveLength(44);
  for(const check of report.checks){
    expect(check.version,check.name).toBe(2);
    expect(check.hasReflection,check.name).toBe(true);
    expect(check.errors,check.name).toEqual([]);
  }
});

test('Rust reports actual shader compilation errors rather than claiming success',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const errors=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();
    if(!adapter)throw Error('WebGPU adapter required');
    const device=await adapter.requestDevice();
    try{
      const fixture=Reflect.get(window,'shaderFixture');
      device.pushErrorScope('validation');
      const module=fixture.shaderModule(device,{wgsl:'this is deliberately invalid WGSL'},'invalid-test');
      const errors=await fixture.compilationErrors(module);
      const validation=await device.popErrorScope();
      if(!validation)throw Error('The actual device must reject the invalid module');
      return errors;
    }finally{device.destroy();}
  });
  expect(errors.length).toBeGreaterThan(0);
  expect(errors.join('\n')).toMatch(/error|expected|unexpected/i);
});

test('Rust fullscreen encoding draws the original copy effect with exact pixel orientation',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();
    if(!adapter)throw Error('WebGPU adapter required for pixel verification');
    const device=await adapter.requestDevice();
    const source=device.createTexture({size:[2,2],format:'rgba8unorm',usage:GPUTextureUsage.TEXTURE_BINDING|GPUTextureUsage.COPY_DST});
    const output=device.createTexture({size:[2,2],format:'rgba8unorm',usage:GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.COPY_SRC});
    const readback=device.createBuffer({size:512,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
    try{
      device.pushErrorScope('validation');
      const fixture=Reflect.get(window,'shaderFixture');
      const shader=Object.entries(fixture.sources).find(([name])=>name.endsWith('/shared/presentation/copy-linear.wgsl'))![1];
      const fragment=fixture.shaderModule(device,shader,'copy-linear');
      const vertex=fixture.fullscreenVertex(device);
      const pipeline=await fixture.fullscreenPipeline(device,vertex,fragment,'rgba8unorm','pixel-copy');
      const pixels=new Uint8Array([255,0,0,255, 0,255,0,255, 0,0,255,255, 85,170,34,255]);
      device.queue.writeTexture({texture:source},pixels,{bytesPerRow:8},[2,2]);
      const bindings=device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:source.createView()}]});
      const encoder=device.createCommandEncoder();
      fixture.encodeFullscreen(encoder,pipeline,bindings,output.createView(),false);
      encoder.copyTextureToBuffer({texture:output},{buffer:readback,bytesPerRow:256},[2,2]);
      device.queue.submit([encoder.finish()]);
      await readback.mapAsync(GPUMapMode.READ);
      const bytes=new Uint8Array(readback.getMappedRange());
      const actual=[...bytes.slice(0,8),...bytes.slice(256,264)];
      readback.unmap();
      const error=await device.popErrorScope();
      return {actual,expected:[...pixels],error:error?.message??null};
    }finally{
      readback.destroy();source.destroy();output.destroy();device.destroy();
    }
  });
  expect(report.error).toBeNull();
  expect(report.actual).toEqual(report.expected);
});
