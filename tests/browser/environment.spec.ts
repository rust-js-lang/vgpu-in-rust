import {expect,test} from '@playwright/test';

test('Rust bakes both authored HDR environments and every mip exactly like upstream',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();
    if(!adapter)throw Error('A real WebGPU adapter is required');
    const device=await adapter.requestDevice();
    const fixture=Reflect.get(window,'shaderFixture');
    const reference=await fixture.referenceEnvironment();
    const gpu=await reference.initFromDevice(device);
    const vertex=fixture.fullscreenVertex(device);
    const probe=device.createShaderModule({code:`
      @group(0) @binding(0) var source: texture_2d<f32>;
      @fragment fn fs_main(@builtin(position) position:vec4f)->@location(0) vec4f {
        return textureLoad(source,vec2i(position.xy),0);
      }`});
    const pipeline=await fixture.fullscreenPipeline(device,vertex,probe,'rgba16float',1,'environment-readback');
    async function pixels(texture:GPUTexture,level:number){
      const width=1024>>level,height=512>>level;
      const target=fixture.createRenderTarget(device,[width,height],'rgba16float',1,'environment-probe');
      const bytesPerRow=Math.ceil(width*8/256)*256;
      const buffer=device.createBuffer({size:bytesPerRow*height,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
      try{
        const bindings=device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{
          binding:0,resource:texture.createView({baseMipLevel:level,mipLevelCount:1}),
        }]});
        const encoder=device.createCommandEncoder();
        const pass=encoder.beginRenderPass({colorAttachments:[fixture.RenderTarget.colorAttachment(target,false,[0,0,0,1])]});
        fixture.encodeFullscreen(pass,pipeline,bindings);
        pass.end();
        encoder.copyTextureToBuffer({texture:target.texture},{buffer,bytesPerRow},[width,height]);
        device.queue.submit([encoder.finish()]);
        await buffer.mapAsync(GPUMapMode.READ);
        const mapped=new Uint8Array(buffer.getMappedRange());
        const packed=new Uint8Array(width*height*8);
        for(let row=0;row<height;row++)packed.set(mapped.subarray(row*bytesPerRow,row*bytesPerRow+width*8),row*width*8);
        buffer.unmap();
        return new Uint16Array(packed.buffer);
      }finally{buffer.destroy();fixture.RenderTarget.destroy(target);}
    }
    const activity={textures:0,buffers:0,submissions:0,uploads:0,samplers:0,pipelines:0};
    const makeTexture=device.createTexture.bind(device),makeBuffer=device.createBuffer.bind(device),submit=device.queue.submit.bind(device.queue);
    device.createTexture=(...args)=>{activity.textures++;return makeTexture(...args);};
    device.createBuffer=(...args)=>{activity.buffers++;return makeBuffer(...args);};
    device.queue.submit=(...args)=>{activity.submissions++;return submit(...args);};
    const write=device.queue.writeBuffer.bind(device.queue),makeSampler=device.createSampler.bind(device),makePipeline=device.createRenderPipelineAsync.bind(device);
    device.queue.writeBuffer=(...args)=>{activity.uploads++;return write(...args);};
    device.createSampler=(...args)=>{activity.samplers++;return makeSampler(...args);};
    device.createRenderPipelineAsync=(...args)=>{activity.pipelines++;return makePipeline(...args);};
    const cache:Array<{before:typeof activity,after:typeof activity,prepared:boolean}>=[];
    const levels:Array<{debug:boolean,level:number,mismatches:number,nonzero:number,nonfinite:number}>=[];
    try{
      device.pushErrorScope('validation');
      for(const debug of [false,true]){
        const label=debug?'debug':'studio';
        const made=fixture.createEnvironmentTexture(device,label,debug);
        if(made.TAG==='Err')throw Error(made._0);
        const actual=made._0;
        const expected=reference.createEnvironmentTexture(gpu,'reference-'+label,debug);
        try{
          const actualSampler=fixture.createEnvironmentSampler(device);
          const result=await fixture.EnvironmentTexture.prepare(actual,device,actualSampler);
          if(result.TAG==='Err')throw Error(result._0);
          const before={...activity};
          const again=await fixture.EnvironmentTexture.prepare(actual,device,actualSampler);
          if(again.TAG==='Err')throw Error(again._0);
          cache.push({before,after:{...activity},prepared:actual.prepared});
          await reference.prepareEnvironmentTexture(gpu,expected,reference.createEnvironmentSampler(gpu));
          for(let level=0;level<8;level++){
            const a=await pixels(actual.texture,level),b=await pixels(expected.texture.gpu,level);
            let mismatches=0,nonzero=0,nonfinite=0;
            for(let index=0;index<a.length;index++){
              if(a[index]!==b[index])mismatches++;
              if(index%4!==3&&a[index]!==0)nonzero++;
              if((a[index]&0x7c00)===0x7c00)nonfinite++;
            }
            levels.push({debug,level,mismatches,nonzero,nonfinite});
          }
        }finally{fixture.EnvironmentTexture.destroy(actual);reference.destroyEnvironmentTexture(expected);}
      }
      return {levels,cache,error:(await device.popErrorScope())?.message??null};
    }finally{gpu.dispose();device.destroy();}
  });
  expect(report.error).toBeNull();
  expect(report.levels).toHaveLength(16);
  for(const cached of report.cache){expect(cached.prepared).toBe(true);expect(cached.after).toEqual(cached.before);}
  for(const level of report.levels){
    expect(level.mismatches,JSON.stringify(level)).toBe(0);
    expect(level.nonzero,JSON.stringify(level)).toBeGreaterThan(0);
    expect(level.nonfinite,JSON.stringify(level)).toBe(0);
  }
});


test('Rust MSAA targets retain identity on unchanged sizes, resize and resolve correctly, and release both textures',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('A real WebGPU adapter is required');
    const device=await adapter.requestDevice(),fixture=Reflect.get(window,'shaderFixture');
    const target=fixture.createRenderTarget(device,[4,4],'rgba8unorm',4,'resize-test');
    let destroyed=0;
    function track(texture:GPUTexture){const original=texture.destroy.bind(texture);texture.destroy=()=>{destroyed++;original();};}
    track(target.texture);track(target.multisampled);
    const old={texture:target.texture,view:target.view,msaa:target.multisampled,msaaView:target.multisampledView};
    const unchanged=fixture.RenderTarget.resize(target,device,[4,4]);
    const retained=target.texture===old.texture&&target.view===old.view&&target.multisampled===old.msaa&&target.multisampledView===old.msaaView;
    const resized=fixture.RenderTarget.resize(target,device,[8,4]);
    const oldDestroyed=destroyed;
    track(target.texture);track(target.multisampled);
    const buffer=device.createBuffer({size:256*4,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
    try{
      device.pushErrorScope('validation');
      const attachment=fixture.RenderTarget.colorAttachment(target,false,[0.25,0.5,0.75,1]);
      const resolved=attachment.view===target.multisampledView&&attachment.resolveTarget===target.view;
      const encoder=device.createCommandEncoder();
      encoder.beginRenderPass({colorAttachments:[attachment]}).end();
      encoder.copyTextureToBuffer({texture:target.texture},{buffer,bytesPerRow:256},[8,4]);
      device.queue.submit([encoder.finish()]);
      await buffer.mapAsync(GPUMapMode.READ);
      const bytes=new Uint8Array(buffer.getMappedRange()),pixels:number[]=[];
      for(let row=0;row<4;row++)pixels.push(...bytes.subarray(row*256,row*256+8*4));
      buffer.unmap();
      fixture.RenderTarget.destroy(target);
      return {unchanged,retained,resized,oldDestroyed,destroyed,resolved,pixels,error:(await device.popErrorScope())?.message??null};
    }finally{buffer.destroy();device.destroy();}
  });
  expect(report.error).toBeNull();expect(report.unchanged).toBe(false);expect(report.retained).toBe(true);
  expect(report.resized).toBe(true);expect(report.resolved).toBe(true);
  expect(report.oldDestroyed).toBe(2);expect(report.destroyed).toBe(4);
  expect(report.pixels).toEqual(Array.from({length:32},()=>[64,128,191,255]).flat());
});
