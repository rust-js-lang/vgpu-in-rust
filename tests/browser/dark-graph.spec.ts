import {expect,test} from '@playwright/test';

for(const low of [false,true])for(const packed of [false,true])test('complete Rust dark graph matches the original pipeline including cached presentation; low='+low+', packed='+packed,async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async({low,packed})=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice({requiredFeatures:packed&&adapter.features.has('rg11b10ufloat-renderable')?['rg11b10ufloat-renderable']:[]}),f=Reflect.get(window,'shaderFixture'),ref=await f.referenceEnvironment(),gpu=await ref.initFromDevice(device);
    device.pushErrorScope('validation');let size=[128,128];const layout=f.lightMeshLayout(low?64:128,low?12:24);
    const actualRuntime=f.createPrismRuntime(device,size,'actual',layout,false),expectedRuntime=ref.createPrismRuntime(gpu,size,'expected',{lightMeshLayout:layout});
    const output=f.createRenderTarget(device,size,'rgba8unorm',1,'output'),expectedOutput=ref.target(gpu,{size,format:'rgba8unorm'});
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    const actual=value(await f.createDarkGraph(device,actualRuntime,{outputFormat:'rgba8unorm',lowQuality:low,sampleCount:4}));
    const expected=ref.createDarkPipeline(expectedRuntime,{quality:low?'low':'high'});await expected.prepare(expectedOutput);
    const comparisons:Array<{view:string,stage:string,different:number,nonzero:number}>=[];
    async function pixels(texture:GPUTexture,bytes:number){
      const row=Math.ceil(texture.width*bytes/256)*256,buffer=device.createBuffer({size:row*texture.height,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});
      try{
        const encoder=device.createCommandEncoder();encoder.copyTextureToBuffer({texture},{buffer,bytesPerRow:row},size);device.queue.submit([encoder.finish()]);
        await buffer.mapAsync(GPUMapMode.READ);const result=new Uint8Array(buffer.getMappedRange()).slice();buffer.unmap();return result;
      }finally{buffer.destroy();}
    }
    async function compare(view:string,stage:string,a:GPUTexture,b:GPUTexture,bytes:number){
      const actual=await pixels(a,bytes),expected=await pixels(b,bytes);let different=0,nonzero=0;
      for(let index=0;index<actual.length;index++){if(actual[index]!==expected[index])different++;if(index%bytes<bytes*3/4&&actual[index]>0)nonzero++;}
      comparisons.push({view,stage,different,nonzero});
    }
    try{
      for(const view of ['glass','back','wall','caustic']){
        const controls=f.defaultPrismControls();controls.view=view;controls.wallColor='#305070';controls.postprocess.bloomRadius=1.4;
        f.PrismRuntime.setControls(actualRuntime,device,controls);ref.setRuntimeControls(expectedRuntime,controls);
        f.PrismRuntime.setOrbit(actualRuntime,0.35,-0.2);ref.setRuntimeOrbit(expectedRuntime,0.35,-0.2);
        f.PrismRuntime.setLampAim(actualRuntime,device,0.58,0.43);ref.setRuntimeLampAim(expectedRuntime,0.58,0.43);
        const frame={time:3.2,updateScene:true,revealProgress:1,beamWidthReveal:0.65};
        value(f.DarkGraph.render(actual,device,actualRuntime,output.view,frame));
        expected.bind(frame.time,frame);ref.frame(gpu,(current:any)=>expected.render(current,expectedOutput,frame));
        await compare(view,'background',actual.background.texture,expected.targets.backdropHDR.color.gpu,8);
        await compare(view,'scene',actual.scene.texture,expected.targets.sceneHDR.color.gpu,8);
        await compare(view,'presentation',actual.postprocess.presentation.texture,expected.targets.presentationLDR.color.gpu,4);
        await compare(view,'output',output.texture,expectedOutput.color.gpu,4);
      }
      const controls=f.defaultPrismControls();f.PrismRuntime.setControls(actualRuntime,device,controls);ref.setRuntimeControls(expectedRuntime,controls);
      let retained=true;const presentation=actual.postprocess.presentation.texture;
      for(const [time,updateScene,revealProgress]of [[4,true,1],[5,false,0.25],[6,false,0],[7,false,1]] as const){
        const frame={time,updateScene,revealProgress,beamWidthReveal:1};
        value(f.DarkGraph.render(actual,device,actualRuntime,output.view,frame));expected.bind(time,frame);ref.frame(gpu,(current:any)=>expected.render(current,expectedOutput,frame));
        await compare('cached-'+time,'output',output.texture,expectedOutput.color.gpu,4);retained&&=actual.postprocess.presentation.texture===presentation;
      }

      function inventory(){
        const post=actual.postprocess;
        const draws=[actual.light,actual.back,actual.front,actual.copy,actual.dust,post.extract,post.composite,post.present,post.copy,...post.levels.flatMap((level:any)=>[level.blurHorizontal,level.blurVertical]),...(post.downsample?[post.downsample]:[])];
        return {pipelines:draws.map((draw:any)=>draw.pipeline),buffers:draws.flatMap((draw:any)=>draw.values.uniforms.map((uniform:any)=>uniform.resource.buffer))};
      }
      const before=inventory();const bundle=actual.backdropBundle;
      let compiles=0;const compile=device.createRenderPipelineAsync.bind(device);device.createRenderPipelineAsync=(descriptor)=>{compiles++;return compile(descriptor);};
      size=[96,144];f.PrismRuntime.resize(actualRuntime,device,size);ref.resizeRuntime(expectedRuntime,size);
      const resized=value(f.DarkGraph.resize(actual,device,actualRuntime));const after=inventory();
      const retainedResources=before.pipelines.every((pipeline:any,index:number)=>pipeline===after.pipelines[index])&&before.buffers.every((buffer:any,index:number)=>buffer===after.buffers[index])&&bundle===actual.backdropBundle;
      const resizeCompiles=compiles;
      f.RenderTarget.resize(output,device,size);expectedOutput.resize(size);expected.resize(size);
      const resizedFrame={time:8,updateScene:false,revealProgress:0.7,beamWidthReveal:0.8};
      value(f.DarkGraph.render(actual,device,actualRuntime,output.view,resizedFrame));expected.bind(8,resizedFrame);ref.frame(gpu,(current:any)=>expected.render(current,expectedOutput,resizedFrame));
      await compare('resized','background',actual.background.texture,expected.targets.backdropHDR.color.gpu,8);
      await compare('resized','scene',actual.scene.texture,expected.targets.sceneHDR.color.gpu,8);
      await compare('resized','output',output.texture,expectedOutput.color.gpu,4);
      for(const [wireframe,lightWireframe]of [[true,false],[false,true],[true,true]]){
        const controls=f.defaultPrismControls();controls.wireframe=wireframe;controls.lightWireframe=lightWireframe;
        f.PrismRuntime.setControls(actualRuntime,device,controls);ref.setRuntimeControls(expectedRuntime,controls);
        value(await f.DarkGraph.prepareWireframes(actual,device,actualRuntime));await expected.prepare(expectedOutput);
        const frame={time:9,updateScene:true,revealProgress:1,beamWidthReveal:0.65};
        value(f.DarkGraph.render(actual,device,actualRuntime,output.view,frame));expected.bind(9,frame);ref.frame(gpu,(current:any)=>expected.render(current,expectedOutput,frame));
        await compare('wire-'+wireframe+'-'+lightWireframe,'background',actual.background.texture,expected.targets.backdropHDR.color.gpu,8);
        await compare('wire-'+wireframe+'-'+lightWireframe,'scene',actual.scene.texture,expected.targets.sceneHDR.color.gpu,8);
        await compare('wire-'+wireframe+'-'+lightWireframe,'output',output.texture,expectedOutput.color.gpu,4);
      }
      return {comparisons,retained,resized,retainedResources,resizeCompiles,error:(await device.popErrorScope())?.message??null};
    }finally{
      f.DarkGraph.destroy(actual);f.PrismRuntime.destroy(actualRuntime);f.RenderTarget.destroy(output);
      expected.destroy();ref.destroyPrismRuntime(expectedRuntime);expectedOutput.destroy?.();gpu.dispose();device.destroy();
    }
  },{low,packed});
  expect(report.error).toBeNull();expect(report.retained).toBe(true);expect(report.resized).toBe(true);expect(report.retainedResources).toBe(true);expect(report.resizeCompiles).toBe(0);
  for(const comparison of report.comparisons){expect(comparison.different,JSON.stringify(comparison)).toBe(0);if(comparison.view==='cached-6')expect(comparison.nonzero).toBe(0);else expect(comparison.nonzero,JSON.stringify(comparison)).toBeGreaterThan(0);}
});

test('late dark graph construction failure releases every graph allocation and preserves the shared runtime for retry',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture');
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    const runtime=f.createPrismRuntime(device,[64,64],'rollback',f.lightMeshLayout(64,12),false);
    const ready=value(f.PrismRuntime.prepareEnvironment(runtime,device));await ready;
    const output=f.createRenderTarget(device,[64,64],'rgba8unorm',1,'output');
    const made:Array<{label:string,destroyed:number}>=[];
    for(const method of ['createTexture','createBuffer'] as const){
      const original=device[method].bind(device);Reflect.set(device,method,(descriptor:any)=>{
        const resource=original(descriptor),record={label:descriptor.label??'',destroyed:0};made.push(record);
        const destroy=resource.destroy.bind(resource);resource.destroy=()=>{record.destroyed++;destroy();};return resource;
      });
    }
    const compile=device.createRenderPipelineAsync.bind(device);
    device.createRenderPipelineAsync=descriptor=>descriptor.label?.endsWith('.dust')?Promise.reject(Error('injected dust failure')):compile(descriptor);
    const failed=await f.createDarkGraph(device,runtime,{outputFormat:'rgba8unorm',lowQuality:true,sampleCount:4});
    const rolledBack=made.length>5&&made.every(record=>record.destroyed===1),failureCount=made.length;
    const sharedReady=value(f.PrismRuntime.prepareEnvironment(runtime,device))===ready&&runtime.studioEnvironment.prepared;
    device.createRenderPipelineAsync=compile;device.pushErrorScope('validation');
    const retry=value(await f.createDarkGraph(device,runtime,{outputFormat:'rgba8unorm',lowQuality:true,sampleCount:4}));
    value(f.DarkGraph.render(retry,device,runtime,output.view,{time:3,updateScene:true,revealProgress:1,beamWidthReveal:1}));
    f.DarkGraph.destroy(retry);
    const retriedWithoutEnvironment=made.slice(failureCount).every(record=>!record.label.includes('environment-'));
    const error=(await device.popErrorScope())?.message??null;
    f.PrismRuntime.destroy(runtime);f.RenderTarget.destroy(output);device.destroy();
    return {failed,rolledBack,sharedReady,retriedWithoutEnvironment,error,completeCleanup:made.every(record=>record.destroyed===1)};
  });
  expect(report.failed).toEqual({TAG:'Err',_0:'injected dust failure'});expect(report.error).toBeNull();
  expect(report.rolledBack).toBe(true);expect(report.sharedReady).toBe(true);expect(report.retriedWithoutEnvironment).toBe(true);expect(report.completeCleanup).toBe(true);
});
