import {expect,test} from '@playwright/test';

for (const layer of ['shadow','accent'] as const) test(
  'Rust light '+layer+' draw matches original indexed geometry and premultiplied composition',
  async({page})=>{
    await page.goto('/shaders.html');
    await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
    const report=await page.evaluate(async(layer)=>{
      const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('A real WebGPU adapter is required');
      const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture');
      const ref=await f.referenceEnvironment(),gpu=await ref.initFromDevice(device);
      function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
      function source(suffix:string){return Object.entries(f.sources).find(([name])=>name.endsWith(suffix))![1];}
      const shader=source(layer==='shadow'?'/shadow/shadow.wgsl':'/glass-accent/glass-accent.wgsl');
      const actual=f.createRenderTarget(device,[128,128],'rgba16float',1,'actual-light-draw');
      const expected=ref.target(gpu,{size:[128,128],format:'rgba16float'});
      const mesh=layer==='shadow'?f.createShadowGeometry(device,'shadow'):f.createPrismGeometry(device,'accent',false);
      const expectedMesh=layer==='shadow'?ref.createPrismShadowGeometry(gpu,'expected-shadow'):ref.prismGeometry(gpu,'expected-accent');
      const environment=value(f.createEnvironmentTexture(device,'accent-environment',false));
      const sampler=f.createEnvironmentSampler(device);
      const allocations:any[]=[];
      async function pixels(texture:GPUTexture){
        const buffer=device.createBuffer({size:128*128*8,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
        try{
          const encoder=device.createCommandEncoder();encoder.copyTextureToBuffer({texture},{buffer,bytesPerRow:1024},[128,128]);
          device.queue.submit([encoder.finish()]);await buffer.mapAsync(GPUMapMode.READ);
          const data=new Uint16Array(buffer.getMappedRange()).slice();buffer.unmap();return data;
        }finally{buffer.destroy();}
      }
      try{
        device.pushErrorScope('validation');
        value(await f.EnvironmentTexture.prepare(environment,device,sampler));
        const pipeline=layer==='shadow'?await f.shadowPipeline(device,shader,'rgba16float','shadow'):
          await f.glassPipeline(device,shader,'rgba16float','Accent',1,'accent');
        const reference=ref.draw(gpu,{shader,geometry:expectedMesh,cull:layer==='shadow'?'none':'back',depth:false,blend:'premultiplied'});
        const cases=[];
        for(const orbit of [[0,0],[0.4,-0.25],[-0.5,0.35]]){
          const view=f.cameraView(1,orbit[0],orbit[1],1.25,48);
          const params=f.glassUniforms(view,f.defaultPrismControls(),'light',false);
          const inputs=layer==='shadow'?[{name:'shadow',value:f.prismShadowUniforms(params.viewProjection)}]:
            [{name:'params',value:params},{name:'accent',value:f.lightGlassAccentUniforms()}];
          const values=value(f.prepareShaderBindings(device,shader,inputs,'light-draw'));allocations.push(values);
          const resources=layer==='shadow'?[]:[
            {name:'studioEnvironment',resource:environment.texture.createView()},
            {name:'debugEnvironment',resource:environment.texture.createView()},
            {name:'environmentSampler',resource:sampler}];
          const group=value(f.ShaderBindings.bind(values,device,pipeline,resources,'light-draw'));
          const clear=[0.7,0.6,0.5,1];
          const encoder=device.createCommandEncoder();
          const pass=encoder.beginRenderPass({colorAttachments:[f.RenderTarget.colorAttachment(actual,false,clear)]});
          if(layer==='shadow')f.encodeShadow(pass,pipeline,group,mesh);else f.encodePrism(pass,pipeline,group,mesh);
          pass.end();device.queue.submit([encoder.finish()]);
          reference.set(layer==='shadow'?{shadow:ref.prismShadowUniforms(params.viewProjection)}:{
            params,accent:ref.LIGHT_GLASS_ACCENT_TUNING,studioEnvironment:environment.texture,
            debugEnvironment:environment.texture,environmentSampler:sampler});
          await reference.compile(expected);
          ref.frame(gpu,(frame:any)=>frame.pass({target:expected,clear},(pass:any)=>pass.draw(reference)));
          const a=await pixels(actual.texture),b=await pixels(expected.color.gpu);
          // The far corner stays at clear color; count changed RGB pixels to reject an empty draw.
          let mismatches=0,changed=0,nonfinite=0;
          for(let i=0;i<a.length;i++){
            if(a[i]!==b[i])mismatches++;
            if(i%4!==3&&a[i]!==a[i%4])changed++;
            if((a[i]&0x7c00)===0x7c00)nonfinite++;
          }
          cases.push({mismatches,changed,nonfinite});
        }
        return {cases,error:(await device.popErrorScope())?.message??null};
      }finally{
        for(const values of allocations)f.ShaderBindings.destroy(values);
        f.RenderTarget.destroy(actual);expected.destroy();expectedMesh.destroy();
        if(layer==='shadow')f.ShadowBuffers.destroy(mesh);else f.PrismBuffers.destroy(mesh);
        f.EnvironmentTexture.destroy(environment);gpu.dispose();device.destroy();
      }
    },layer);
    expect(report.error).toBeNull();
    for(const frame of report.cases){expect(frame.mismatches).toBe(0);expect(frame.nonfinite).toBe(0);expect(frame.changed).toBeGreaterThan(100);}
  });
