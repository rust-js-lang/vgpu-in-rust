import {expect,test} from '@playwright/test';

test('theme and quality controller retains live dark rendering during stale light preparation and matches upstream activation',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  await page.route('**/hero/prism-light/wall-global-light-mask.webp',async route=>{
    const path=await page.evaluate(()=>Reflect.get(window,'shaderFixture').wallMaskUrl);
    const response=await page.request.get(path);await route.fulfill({body:await response.body(),contentType:'image/webp'});
  });
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture'),ref=await f.referenceEnvironment(),gpu=await ref.initFromDevice(device);
    const size=[96,96],layout=f.lightMeshLayout(128,24);
    const runtime=f.createPrismRuntime(device,size,'actual',layout,false),originalRuntime=ref.createPrismRuntime(gpu,size,'expected',{lightMeshLayout:layout});
    const output=f.createRenderTarget(device,size,'rgba8unorm',1,'actual-output'),originalOutput=ref.target(gpu,{size,format:'rgba8unorm'});
    const actualEvents:any[]=[],expectedEvents:any[]=[];
    const actual=f.createPipelineController(device,runtime,{initialMode:'dark',initialQuality:'high',outputFormat:'rgba8unorm',sampleCount:4,onActivate:(mode:string,quality:string)=>actualEvents.push([mode,quality])});
    const original=ref.createPrismPipelineController({runtime:originalRuntime,output:originalOutput,initialMode:'dark',initialQuality:'high',onActivate:(mode:string,quality:string)=>expectedEvents.push([mode,quality])});
    const ready=f.PipelineController.ready(actual),comparisons:number[]=[];
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    async function pixels(texture:GPUTexture){
      const buffer=device.createBuffer({size:512*96,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});
      try{const encoder=device.createCommandEncoder();encoder.copyTextureToBuffer({texture},{buffer,bytesPerRow:512},size);device.queue.submit([encoder.finish()]);
        await buffer.mapAsync(GPUMapMode.READ);const result=new Uint8Array(buffer.getMappedRange()).slice();buffer.unmap();return result;
      }finally{buffer.destroy();}
    }
    async function compare(time:number){
      const frame={time,updateScene:true,revealProgress:1,beamWidthReveal:1};
      value(f.PipelineController.render(actual,output.view,frame));original.pipeline.bind(time,frame);ref.frame(gpu,(current:any)=>original.pipeline.render(current,originalOutput,frame));
      const a=await pixels(output.texture),b=await pixels(originalOutput.color.gpu);let different=0,lit=0;
      for(let i=0;i<a.length;i++){if(a[i]!==b[i])different++;if(i%4!==3&&a[i]>0)lit++;}
      if(lit<100)throw Error('Both controllers must render a nonblank scene');comparisons.push(different);return a;
    }
    let release:()=>void=()=>{};
    const compile=device.createRenderPipelineAsync.bind(device);
    try{
      device.pushErrorScope('validation');await Promise.all([ready,original.ready]);
      const first=await compare(3);
      let actualEntered!:()=>void,expectedEntered!:()=>void;
      const labels:string[]=[];
      const gate=new Promise<void>(resolve=>release=resolve),aEntered=new Promise<void>(resolve=>actualEntered=resolve),bEntered=new Promise<void>(resolve=>expectedEntered=resolve);
      device.createRenderPipelineAsync=descriptor=>{
        labels.push(descriptor.label??"");
        if(descriptor.label==='light.present'){actualEntered();return gate.then(()=>compile(descriptor));}
        if(descriptor.label==='expected.light.present.pipeline'){expectedEntered();return gate.then(()=>compile(descriptor));}
        return compile(descriptor);
      };
      const aLight=f.PipelineController.setMode(actual,'light'),bLight=original.setMode('light');
      let gateTimeout:ReturnType<typeof setTimeout>;
      try{await Promise.race([Promise.all([aEntered,bEntered]),new Promise((_,reject)=>{gateTimeout=setTimeout(()=>reject(Error("Missing gate: "+JSON.stringify(labels))),5000);})]);}finally{clearTimeout(gateTimeout!);}
      const keptActive=f.PipelineController.activeMode(actual)==='dark'&&original.pipeline.mode==='dark';
      const unchangedLayout=runtime.state.lightMeshLayout.samples===128;
      f.PrismRuntime.setOrbit(runtime,0.5,-0.25);ref.setRuntimeOrbit(originalRuntime,0.5,-0.25);
      const during=await compare(4);const interactive=during.some((byte:number,index:number)=>byte!==first[index]);
      const aLow=f.PipelineController.setQuality(actual,'low'),bLow=original.setQuality('low');
      const aBack=f.PipelineController.setMode(actual,'dark'),bBack=original.setMode('dark');
      const singleTask=aLight===aLow&&aLow===aBack;
      release();await Promise.all([aLight,bLight,aLow,bLow,aBack,bBack]);device.createRenderPipelineAsync=compile;
      await compare(5);
      const lowActivated=f.PipelineController.activeQuality(actual)==='low'&&runtime.state.lightMeshLayout.samples===64;
      await Promise.all([f.PipelineController.setMode(actual,'light'),original.setMode('light')]);await compare(6);
      const initialIdentity=f.PipelineController.ready(actual)===ready;
      return {comparisons,actualEvents,expectedEvents,keptActive,unchangedLayout,interactive,singleTask,lowActivated,initialIdentity,error:(await device.popErrorScope())?.message??null};
    }finally{
      release();device.createRenderPipelineAsync=compile;
      await f.PipelineController.destroy(actual);await original.destroy();f.PrismRuntime.destroy(runtime);ref.destroyPrismRuntime(originalRuntime);
      f.RenderTarget.destroy(output);originalOutput.destroy();gpu.dispose();device.destroy();
    }
  });
  expect(report.error).toBeNull();expect(report.comparisons).toEqual([0,0,0,0]);expect(report.actualEvents).toEqual(report.expectedEvents);
  expect(report.actualEvents).toEqual([['dark','high'],['dark','low'],['light','low']]);
  for(const field of ['keptActive','unchangedLayout','interactive','singleTask','lowActivated','initialIdentity'] as const)expect(report[field],field).toBe(true);
});

test('disposing during light preparation waits before shared runtime cleanup and destroys every owned allocation once',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture');
    const made:Array<{label:string,destroyed:number}>=[];
    for(const method of ['createTexture','createBuffer'] as const){const original=device[method].bind(device);Reflect.set(device,method,(descriptor:any)=>{
      const resource=original(descriptor),record={label:descriptor.label??'',destroyed:0};made.push(record);const destroy=resource.destroy.bind(resource);
      resource.destroy=()=>{record.destroyed++;destroy();};return resource;
    });}
    const runtime=f.createPrismRuntime(device,[64,64],'dispose',f.lightMeshLayout(128,24),false);
    let release!:()=>void,entered!:()=>void;const gate=new Promise<void>(resolve=>release=resolve),started=new Promise<void>(resolve=>entered=resolve);
    const compile=device.createRenderPipelineAsync.bind(device);device.createRenderPipelineAsync=descriptor=>{
      if(descriptor.label!=='light.present')return compile(descriptor);entered();return gate.then(()=>compile(descriptor));
    };
    const events:any[]=[];const controller=f.createPipelineController(device,runtime,{initialMode:'light',initialQuality:'high',outputFormat:'rgba8unorm',sampleCount:4,onActivate:(...args:any[])=>events.push(args)});
    await started;
    const cleanup=f.PipelineController.destroy(controller);const sameCleanup=f.PipelineController.destroy(controller)===cleanup;
    let cleaned=false;cleanup.then(()=>cleaned=true);await Promise.resolve();await Promise.resolve();
    const waited=!cleaned&&made.find(record=>record.label==='dispose.light-vertices')?.destroyed===0;
    release();await cleanup;device.createRenderPipelineAsync=compile;
    const noActivation=events.length===0&&f.PipelineController.activeMode(controller)==null;
    await f.PipelineController.setMode(controller,'dark');await f.PipelineController.setQuality(controller,'low');
    f.PrismRuntime.destroy(runtime);device.destroy();
    return {sameCleanup,waited,noActivation,count:made.length,leaks:made.filter(record=>record.destroyed!==1)};
  });
  expect(report.sameCleanup).toBe(true);expect(report.waited).toBe(true);expect(report.noActivation).toBe(true);
  expect(report.count).toBeGreaterThan(10);expect(report.leaks).toEqual([]);
});

for(const switchKind of ['quality','theme'] as const)test('failed '+switchKind+' preparation preserves the active image and live mesh, then retries',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async(switchKind)=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture');
    const runtime=f.createPrismRuntime(device,[64,64],'failure',f.lightMeshLayout(128,24),false);
    const output=f.createRenderTarget(device,[64,64],'rgba8unorm',1,'failure-output');
    const events:any[]=[];const controller=f.createPipelineController(device,runtime,{initialMode:'dark',initialQuality:'high',outputFormat:'rgba8unorm',sampleCount:4,onActivate:(...event:any[])=>events.push(event)});
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    async function image(){
      value(f.PipelineController.render(controller,output.view,{time:3,updateScene:true,revealProgress:1,beamWidthReveal:1}));
      const buffer=device.createBuffer({size:256*64,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});
      try{const encoder=device.createCommandEncoder();encoder.copyTextureToBuffer({texture:output.texture},{buffer,bytesPerRow:256},[64,64]);device.queue.submit([encoder.finish()]);
        await buffer.mapAsync(GPUMapMode.READ);const result=new Uint8Array(buffer.getMappedRange()).slice();buffer.unmap();return result;
      }finally{buffer.destroy();}
    }
    const compile=device.createRenderPipelineAsync.bind(device);
    try{
      await f.PipelineController.ready(controller);const before=await image();
      device.createRenderPipelineAsync=descriptor=>descriptor.label===(switchKind==='quality'?'failure.dust':'light.present')?Promise.reject(Error('injected candidate failure')):compile(descriptor);
      let message='';try{await (switchKind==='quality'?f.PipelineController.setQuality(controller,'low'):f.PipelineController.setMode(controller,'light'));}catch(error:any){message=error.message;}
      const preserved=f.PipelineController.activeMode(controller)==='dark'&&f.PipelineController.activeQuality(controller)==='high'&&runtime.state.lightMeshLayout.samples===128;
      const after=await image();let different=0;for(let index=0;index<before.length;index++)if(before[index]!==after[index])different++;
      device.createRenderPipelineAsync=compile;
      await (switchKind==='quality'?f.PipelineController.setQuality(controller,'low'):f.PipelineController.setMode(controller,'light'));
      const active=f.PipelineController.activeMode(controller),quality=f.PipelineController.activeQuality(controller);await image();
      return {message,preserved,different,active,quality,events};
    }finally{
      device.createRenderPipelineAsync=compile;await f.PipelineController.destroy(controller);f.PrismRuntime.destroy(runtime);f.RenderTarget.destroy(output);device.destroy();
    }
  },switchKind);
  expect(report.message).toBe('injected candidate failure');expect(report.preserved).toBe(true);expect(report.different).toBe(0);
  expect([report.active,report.quality]).toEqual(switchKind==='quality'?['dark','low']:['light','high']);expect(report.events).toHaveLength(2);
});
