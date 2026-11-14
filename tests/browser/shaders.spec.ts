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
      const pipeline=await fixture.fullscreenPipeline(device,vertex,fragment,'rgba8unorm',1,'pixel-copy');
      const pixels=new Uint8Array([255,0,0,255, 0,255,0,255, 0,0,255,255, 85,170,34,255]);
      device.queue.writeTexture({texture:source},pixels,{bytesPerRow:8},[2,2]);
      const bindings=device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:source.createView()}]});
      const encoder=device.createCommandEncoder();
      const pass=encoder.beginRenderPass({colorAttachments:[{view:output.createView(),loadOp:'clear',storeOp:'store'}]});
      fixture.encodeFullscreen(pass,pipeline,bindings);
      pass.end();
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
    const controls=fixture.defaultPrismControls();
    controls.view='caustic';
    const view={viewProjection:[0.5,0,0,0, 0,0.8,0,0, 0,0,0,0, 0,0,0.5,1]};
    const values=fixture.sceneUniforms(view,[2.8,1.6],light,controls,fixture.lightMeshLayout(64,12),1);
    const allocated=fixture.createUniformBuffer(device,layout,values,'spectral-test.uniforms');
    if(allocated.TAG==='Err')throw Error(allocated._0.message);
    const uniformResource=allocated._0;
    const uniforms=uniformResource.buffer;
    const vertices=fixture.createVertexBuffer(device,mesh.vertices,'spectral-test.vertices');
    const output=device.createTexture({size:[256,128],format:'rgba8unorm',usage:GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.COPY_SRC});
    const readback=device.createBuffer({size:256*128*4,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
    try{
      device.pushErrorScope('validation');
      const pipeline=await fixture.lightPipeline(device,shader,'rgba8unorm','spectral-test');
      const bindings=device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:uniforms}}]});
      // Values, allocation, validation and retained uploads all run in Rust.
      function pack(){
        const result=fixture.writeUniformBuffer(uniformResource,device.queue,values);
        if(result.TAG==='Err')throw Error(result._0.message);
      }
      async function frame(){
        pack();
        const encoder=device.createCommandEncoder();
        const pass=encoder.beginRenderPass({colorAttachments:[{view:output.createView(),loadOp:'clear',storeOp:'store'}]});
        fixture.encodeLight(pass,pipeline,bindings,vertices,0,mesh.vertexCount);
        pass.end();
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


test('Rust glass interfaces compile against native mesh attributes in single-sample and 4x MSAA targets',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();
    if(!adapter)throw Error('A real WebGPU adapter is required');
    const device=await adapter.requestDevice();
    const fixture=Reflect.get(window,'shaderFixture');
    try{
      device.pushErrorScope('validation');
      for(const back of [false,true])for(const samples of [1,4]){
        const suffix=back?'/shared/glass/glass-back.wgsl':'/shared/glass/glass.wgsl';
        const shader=Object.entries(fixture.sources).find(([name])=>name.endsWith(suffix))![1];
        await fixture.glassPipeline(device,shader,'rgba16float',back?'Back':'Front',samples,'glass-test');
      }
      return (await device.popErrorScope())?.message??null;
    }finally{device.destroy();}
  });
  expect(report).toBeNull();
});
