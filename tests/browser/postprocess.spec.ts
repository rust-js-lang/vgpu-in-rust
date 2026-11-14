import {expect,test} from '@playwright/test';

for(const low of [false,true])for(const packed of [false,true])test('Rust bloom and display match upstream: '+(low?'low':'high')+', '+(packed?'packed when supported':'HDR fallback'),async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async({low,packed})=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const enabled=packed&&adapter.features.has('rg11b10ufloat-renderable');
    const device=await adapter.requestDevice({requiredFeatures:enabled?['rg11b10ufloat-renderable']:[]});
    device.pushErrorScope('validation');
    const f=Reflect.get(window,'shaderFixture'),ref=await f.referenceEnvironment(),gpu=await ref.initFromDevice(device);
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    function shader(suffix:string){return Object.entries(f.sources).find(([name])=>name.endsWith(suffix))![1];}
    const width=128,height=96;
    const scene=f.createRenderTarget(device,[width,height],'rgba16float',1,'input-HDR');
    const output=f.createRenderTarget(device,[width,height],'rgba8unorm',1,'display');
    const expectedOutput=ref.target(gpu,{size:[width,height],format:'rgba8unorm'});
    const sampler=device.createSampler({minFilter:'linear',magFilter:'linear',addressModeU:'clamp-to-edge',addressModeV:'clamp-to-edge'});
    const seed=device.createShaderModule({code:
      '@fragment fn fs_main(@builtin(position) p:vec4f)->@location(0) vec4f { let stripe=select(0.0,12.0,abs(p.y-0.3*p.x-25.0)<1.0); return vec4f(stripe+0.03*p.x,0.05*p.y,0.2+stripe*0.7,1.0); }'});
    const seedPipeline=await f.fullscreenPipeline(device,f.fullscreenVertex(device),seed,'rgba16float',1,'input');
    const initial=device.createCommandEncoder();const pass=initial.beginRenderPass({colorAttachments:[f.RenderTarget.colorAttachment(scene,false,[0,0,0,1])]});
    pass.setPipeline(seedPipeline);pass.draw(3);pass.end();device.queue.submit([initial.finish()]);
    const graph=value(await f.createDarkPostprocess(device,scene,sampler,{outputFormat:'rgba8unorm',lowQuality:low,threshold:0.1,radius:1.4,strength:0.7,label:'actual'}));
    const count=low?2:4,visible=low?2:3;
    const targets=Array.from({length:count},(_,level)=>{
      const size=[Math.max(1,Math.ceil(width/[2,4,8,16][level])),Math.max(1,Math.ceil(height/[2,4,8,16][level]))];
      const format=enabled&&level<3?'rg11b10ufloat':'rgba16float';
      return {horizontal:ref.target(gpu,{size,format}),vertical:ref.target(gpu,{size,format})};
    });
    const effects:any[]=[];
    function effect(suffix:string,inputs:any){const result=ref.effect(gpu,shader(suffix));result.set(inputs);effects.push(result);return result;}
    const extract=effect('/bloom-extract.wgsl',{sourceTexture:scene.texture,sourceSampler:sampler,params:{threshold:0.1}});
    const blurs=targets.map((target,level)=>{
      const source=level===0||(!low&&level===visible)?target.vertical:targets[level-1].vertical;
      return {
        horizontal:effect(level===0||level===3?'/bloom-blur-paired.wgsl':'/bloom-blur.wgsl',{sourceTexture:source,sourceSampler:sampler,params:ref.bloomBlurUniforms(level,'horizontal',target.horizontal.size)}),
        vertical:effect('/bloom-blur-paired.wgsl',{sourceTexture:target.horizontal,sourceSampler:sampler,params:ref.bloomBlurUniforms(level,'vertical',target.vertical.size)}),
      };
    });
    const downsample=low?null:effect('/particle-light-downsample.wgsl',{sourceTexture:scene.texture,sourceSampler:sampler,params:{sourceTexelSize:[1/width,1/height],sourceToTargetScale:[width/targets[3].vertical.size[0],height/targets[3].vertical.size[1]]}});
    const composite=effect(low?'/bloom-composite-low.wgsl':'/bloom-composite.wgsl',{
      level0Texture:targets[0].vertical,level1Texture:targets[1].vertical,...(low?{}:{level2Texture:targets[2].vertical}),levelSampler:sampler,
      params:{radius:ref.bloomSpread(1.4,0.25,3),factors:low?[1,0.8]:[1,0.8,0.55,0]},
    });
    const presentation=ref.target(gpu,{size:[width,height],format:'rgba8unorm'});
    const present=effect('/dark/passes/presentation/present.wgsl',{sceneTexture:scene.texture,bloomTexture:targets[0].horizontal,bloomSampler:sampler,params:{bloomStrength:low?0.15:0.7}});
    const copy=effect('/copy-presentation.wgsl',{sourceTexture:presentation,params:{backgroundColor:[0,0,0],revealProgress:1}});
    try{
      await Promise.all([extract.compile(targets[0].vertical),...blurs.flatMap((blur,level)=>[blur.horizontal.compile(targets[level].horizontal),blur.vertical.compile(targets[level].vertical)]),...(downsample?[downsample.compile(targets[3].vertical)]:[]),composite.compile(targets[0].horizontal),present.compile(presentation),copy.compile(expectedOutput)]);
      const encoder=device.createCommandEncoder();f.DarkPostprocess.encodeScene(graph,encoder);f.DarkPostprocess.encodeOutput(graph,encoder,output);device.queue.submit([encoder.finish()]);
      ref.frame(gpu,(frame:any)=>{
        function draw(target:any,draw:any){frame.pass({target,clear:[0,0,0,1]},(pass:any)=>pass.draw(draw));}
        draw(targets[0].vertical,extract);
        for(let level=0;level<visible;level++){draw(targets[level].horizontal,blurs[level].horizontal);draw(targets[level].vertical,blurs[level].vertical);}
        if(downsample){draw(targets[3].vertical,downsample);draw(targets[3].horizontal,blurs[3].horizontal);draw(targets[3].vertical,blurs[3].vertical);}
        draw(targets[0].horizontal,composite);draw(presentation,present);draw(expectedOutput,copy);
      });
      const probe=device.createShaderModule({code:'@group(0) @binding(0) var source:texture_2d<f32>; @fragment fn fs_main(@builtin(position) p:vec4f)->@location(0) vec4f { return textureLoad(source,vec2i(p.xy),0); }'});
      const probePipeline=await f.fullscreenPipeline(device,f.fullscreenVertex(device),probe,'rgba32float',1,'read');
      async function pixels(texture:GPUTexture,w:number,h:number){
        const target=f.createRenderTarget(device,[w,h],'rgba32float',1,'probe');
        const stride=Math.ceil(w*16/256)*256,buffer=device.createBuffer({size:stride*h,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});
        try{
          const group=device.createBindGroup({layout:probePipeline.getBindGroupLayout(0),entries:[{binding:0,resource:texture.createView()}]});
          const encoder=device.createCommandEncoder(),pass=encoder.beginRenderPass({colorAttachments:[f.RenderTarget.colorAttachment(target,false,[0,0,0,1])]});
          f.encodeFullscreen(pass,probePipeline,group);pass.end();encoder.copyTextureToBuffer({texture:target.texture},{buffer,bytesPerRow:stride},[w,h]);device.queue.submit([encoder.finish()]);
          await buffer.mapAsync(GPUMapMode.READ);const mapped=new Uint8Array(buffer.getMappedRange()),bytes=new Uint8Array(w*h*16);
          for(let row=0;row<h;row++)bytes.set(mapped.subarray(row*stride,row*stride+w*16),row*w*16);
          buffer.unmap();return new Float32Array(bytes.buffer);
        }finally{buffer.destroy();f.RenderTarget.destroy(target);}
      }
      let mismatches=0,lit=0,nonfinite=0;
      async function compare(a:GPUTexture,b:GPUTexture,w:number,h:number){const x=await pixels(a,w,h),y=await pixels(b,w,h);for(let index=0;index<x.length;index++){if(x[index]!==y[index])mismatches++;if(!Number.isFinite(x[index]))nonfinite++;if(index%4!==3&&x[index]>0)lit++;}}
      for(let level=0;level<count;level++)for(const axis of ['horizontal','vertical']){
        const a=graph.levels[level][axis],b=targets[level][axis];await compare(a.texture,b.color.gpu,a.size[0],a.size[1]);
      }
      await compare(graph.presentation.texture,presentation.color.gpu,width,height);await compare(output.texture,expectedOutput.color.gpu,width,height);
      const retained=graph.presentation.texture;
      for(const reveal of [0,0.25,1]){
        value(f.DarkPostprocess.setReveal(graph,device,reveal));copy.set({params:{backgroundColor:[0,0,0],revealProgress:reveal}});
        const encoder=device.createCommandEncoder();f.DarkPostprocess.encodeOutput(graph,encoder,output);device.queue.submit([encoder.finish()]);
        ref.frame(gpu,(frame:any)=>frame.pass({target:expectedOutput},(pass:any)=>pass.draw(copy)));
        await compare(output.texture,expectedOutput.color.gpu,width,height);
      }
      return {mismatches,lit,nonfinite,retained:retained===graph.presentation.texture,error:(await device.popErrorScope())?.message??null};
    }finally{
      f.DarkPostprocess.destroy(graph);f.RenderTarget.destroy(scene);f.RenderTarget.destroy(output);
      for(const level of targets){level.horizontal.destroy?.();level.vertical.destroy?.();}
      presentation.destroy?.();expectedOutput.destroy?.();for(const effect of effects)effect.destroy?.();gpu.dispose();device.destroy();
    }
  },{low,packed});
  expect(report.error).toBeNull();expect(report.mismatches).toBe(0);expect(report.nonfinite).toBe(0);expect(report.lit).toBeGreaterThan(100);expect(report.retained).toBe(true);
});

test('a rejected native pipeline releases all partially constructed graph resources',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture');
    const scene=f.createRenderTarget(device,[64,64],'rgba16float',1,'input');
    const sampler=device.createSampler({minFilter:'linear',magFilter:'linear'});
    const made={textures:0,buffers:0},destroyed={textures:0,buffers:0};
    for(const [method,key] of [['createTexture','textures'],['createBuffer','buffers']] as const){
      const original=device[method].bind(device);
      Reflect.set(device,method,(descriptor:any)=>{
        const result=original(descriptor);made[key]++;
        const destroy=result.destroy.bind(result);result.destroy=()=>{destroyed[key]++;destroy();};return result;
      });
    }
    let pipelines=0;const compile=device.createRenderPipelineAsync.bind(device);
    device.createRenderPipelineAsync=(descriptor)=>++pipelines===4?Promise.reject(new Error('injected pipeline failure')):compile(descriptor);
    try{
      const result=await f.createDarkPostprocess(device,scene,sampler,{outputFormat:'rgba8unorm',lowQuality:false,threshold:0.1,radius:1.4,strength:0.7,label:'failure'});
      return {result,made,destroyed,pipelines};
    }finally{f.RenderTarget.destroy(scene);device.destroy();}
  });
  expect(report.result).toEqual({TAG:'Err',_0:'injected pipeline failure'});
  expect(report.pipelines).toBe(4);expect(report.made.buffers).toBeGreaterThan(0);expect(report.made.textures).toBeGreaterThan(0);
  expect(report.destroyed).toEqual(report.made);
});
