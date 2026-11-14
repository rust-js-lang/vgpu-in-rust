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


test('Rust additive light pass rasterizes the generated spectral mesh and obeys the reveal uniform',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();
    if(!adapter)throw Error('A real WebGPU adapter is required');
    const device=await adapter.requestDevice();
    const fixture=Reflect.get(window,'shaderFixture');
    const shader=Object.entries(fixture.sources).find(([name])=>name.endsWith('/dark/passes/light/light.wgsl'))![1] as any;
    const layout=shader.reflection.bindings.find((binding:any)=>binding.group===0&&binding.binding===0).layout;
    const light=fixture.lampForIncidence(60,0.025,0.5);
    const mesh=fixture.buildLightMesh({light,dispersion:fixture.STYLIZED_DISPERSION,wallHalfExtent:[2.8,1.6],samples:64,beamSlices:12});
    const vertices=device.createBuffer({size:mesh.vertices.byteLength,usage:GPUBufferUsage.VERTEX|GPUBufferUsage.COPY_DST});
    const uniforms=device.createBuffer({size:layout.size,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST});
    const output=device.createTexture({size:[256,128],format:'rgba8unorm',usage:GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.COPY_SRC});
    const readback=device.createBuffer({size:256*128*4,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
    try{
      device.pushErrorScope('validation');
      const pipeline=await fixture.lightPipeline(device,shader,'rgba8unorm','spectral-test');
      const bindings=device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:uniforms}}]});
      device.queue.writeBuffer(vertices,0,mesh.vertices);
      // Test-only reflected data packing; application uniform ownership is ported separately.
      const values:Record<string,number|number[]>={viewProjection:[0.5,0,0,0, 0,0.8,0,0, 0,0,0,0, 0,0,0.5,1],wallHalfExtent:[2.8,1.6],inputBeamDirection:light.direction,wallColor:[0,0,0],causticOnly:1,lightPlaneZ:0,lightWhiteQuads:12,lightBeamSlices:12,lightSpectralSamples:64,lightInternalQuads:64*12*4,lightInternalSegments:4,lightOpacity:1,lightEdgeFalloff:16,rainbowFalloffRate:3.8,rainbowFalloffPower:3.7,beamWidthReveal:1};
      const bytes=new ArrayBuffer(layout.size),view=new DataView(bytes);
      function pack(){
        for(const member of layout.members){
          const value=values[member.name];
          if(value===undefined)throw Error('Missing test uniform '+member.name);
          if(Array.isArray(value)){
            const type=member.layout.type;
            value.forEach((component,index)=>{
              const offset=type.kind==='matrix'?Math.floor(index/type.rows)*member.layout.stride+(index%type.rows)*4:index*4;
              view.setFloat32(member.offset+offset,component,true);
            });
          }else if(member.type.name==='u32')view.setUint32(member.offset,value,true);
          else view.setFloat32(member.offset,value,true);
        }
        device.queue.writeBuffer(uniforms,0,bytes);
      }
      async function frame(){
        pack();
        const encoder=device.createCommandEncoder();
        fixture.encodeLight(encoder,pipeline,bindings,vertices,mesh.vertexCount,output.createView(),false);
        encoder.copyTextureToBuffer({texture:output},{buffer:readback,bytesPerRow:1024},[256,128]);
        device.queue.submit([encoder.finish()]);
        await readback.mapAsync(GPUMapMode.READ);
        const pixels=new Uint8Array(readback.getMappedRange());
        let lit=0,red=0,blue=0,maximum=0;
        for(let i=0;i<pixels.length;i+=4){
          const [r,g,b]=[pixels[i],pixels[i+1],pixels[i+2]];
          maximum=Math.max(maximum,r,g,b);
          if(Math.max(r,g,b)>4)lit++;
          if(r>g+10&&r>b+10)red++;
          if(b>r+10&&b>g+10)blue++;
        }
        readback.unmap();
        return {lit,red,blue,maximum};
      }
      const visible=await frame();
      values.beamWidthReveal=0;
      const hidden=await frame();
      const error=await device.popErrorScope();
      return {visible,hidden,error:error?.message??null};
    }finally{readback.destroy();output.destroy();uniforms.destroy();vertices.destroy();device.destroy();}
  });
  expect(report.error).toBeNull();
  expect(report.visible.lit).toBeGreaterThan(50);
  expect(report.visible.red).toBeGreaterThan(0);
  expect(report.visible.blue).toBeGreaterThan(0);
  expect(report.visible.maximum).toBeGreaterThan(50);
  expect(report.hidden.lit).toBe(0);
});
