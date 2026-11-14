import {expect,test} from '@playwright/test';

for(const reporterThrows of [false,true])test('missing WebGPU reports once and cleans up; reporter throws='+reporterThrows,async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async(reporterThrows)=>{
    const f=Reflect.get(window,'shaderFixture'),canvas=document.createElement('canvas');
    Object.defineProperty(navigator,'gpu',{configurable:true,value:undefined});
    const errors:string[]=[];const renderer=f.createRenderer({canvas,initialMode:'dark',initialQuality:'high',onError:(error:any)=>{errors.push(error.message);if(reporterThrows)throw Error('reporter failure');}});
    let rejected='';try{await f.BrowserRenderer.ready(renderer);}catch(error:any){rejected=error.message;}
    const cleanup=f.BrowserRenderer.dispose(renderer);const identity=cleanup===f.BrowserRenderer.dispose(renderer);await cleanup;
    delete (navigator as any).gpu;
    return {errors,rejected,identity,status:f.BrowserRenderer.status(renderer)};
  },reporterThrows);
  expect(report.errors).toEqual(['WebGPU is unavailable.']);expect(report.rejected).toBe('WebGPU is unavailable.');
  expect(report.identity).toBe(true);expect(report.status.disposed).toBe(true);expect(report.status.running).toBe(false);
});

for(const failure of ['context','configure'] as const)test('failed '+failure+' initialization releases the device and preserves canvas state',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async(failure)=>{
    const f=Reflect.get(window,'shaderFixture'),canvas=document.createElement('canvas');canvas.width=333;canvas.height=111;canvas.style.cssText='width:128px;height:128px';document.body.append(canvas);
    let destroyed=0,unconfigured=0;const request=navigator.gpu.requestAdapter.bind(navigator.gpu);
    navigator.gpu.requestAdapter=async options=>{const adapter=await request(options);if(!adapter)return adapter;const create=adapter.requestDevice.bind(adapter);adapter.requestDevice=async options=>{const device=await create(options),destroy=device.destroy.bind(device);device.destroy=()=>{destroyed++;destroy();};return device;};return adapter;};
    const injected=Error('injected configure failure');
    if(failure==='context')canvas.getContext('2d');else{const context=canvas.getContext('webgpu')!,unconfigure=context.unconfigure.bind(context);context.configure=()=>{throw injected;};context.unconfigure=()=>{unconfigured++;unconfigure();};}
    const errors:any[]=[];const renderer=f.createRenderer({canvas,initialMode:'dark',initialQuality:'high',onError:(error:any)=>errors.push(error)});
    let message='';let sameError=false;try{await f.BrowserRenderer.ready(renderer);}catch(error:any){message=error.message;sameError=error===injected;}
    await f.BrowserRenderer.dispose(renderer);navigator.gpu.requestAdapter=request;
    return {message,sameError,destroyed,unconfigured,errorCount:errors.length,size:[canvas.width,canvas.height],status:f.BrowserRenderer.status(renderer)};
  },failure);
  expect(report.destroyed).toBe(1);expect(report.errorCount).toBe(1);expect(report.status.disposed).toBe(true);expect(report.size).toEqual([333,111]);
  expect(report.message).toBe(failure==='context'?'Canvas WebGPU context is unavailable.':'injected configure failure');
  if(failure==='configure'){expect(report.sameError).toBe(true);expect(report.unconfigured).toBe(1);}
});

test('optional device feature rejection retries without features',async({page})=>{
  await page.emulateMedia({reducedMotion:'reduce'});await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const f=Reflect.get(window,'shaderFixture'),canvas=document.createElement('canvas');canvas.style.cssText='position:absolute;left:0;top:0;width:64px;height:64px';document.body.append(canvas);
    const request=navigator.gpu.requestAdapter.bind(navigator.gpu),requests:any[]=[];
    navigator.gpu.requestAdapter=async options=>{
      const adapter=await request(options);if(!adapter)return adapter;
      Object.defineProperty(adapter,'features',{configurable:true,value:new Set(['rg11b10ufloat-renderable'])});
      const create=adapter.requestDevice.bind(adapter);adapter.requestDevice=async options=>{requests.push(options?.requiredFeatures??[]);if(options?.requiredFeatures?.length)throw Error('injected feature rejection');return create(options);};return adapter;
    };
    const errors:string[]=[];const renderer=f.createRenderer({canvas,initialMode:'dark',initialQuality:'high',onError:(error:any)=>errors.push(error.message)});
    try{await f.BrowserRenderer.ready(renderer);return {requests,errors,status:f.BrowserRenderer.status(renderer)};}
    finally{await f.BrowserRenderer.dispose(renderer);navigator.gpu.requestAdapter=request;}
  });
  expect(report.requests).toEqual([['rg11b10ufloat-renderable'],[]]);expect(report.errors).toEqual([]);expect(report.status.disposed).toBe(false);
});

test('dispose during a device request waits for the late device and never configures a canvas',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const f=Reflect.get(window,'shaderFixture'),canvas=document.createElement('canvas');let release!:()=>void,entered!:()=>void;
    const gate=new Promise<void>(resolve=>release=resolve),started=new Promise<void>(resolve=>entered=resolve);
    const request=navigator.gpu.requestAdapter.bind(navigator.gpu);let destroyed=0,contexts=0;const get=canvas.getContext.bind(canvas);canvas.getContext=((...args:any[])=>{contexts++;return get(...args);}) as any;
    navigator.gpu.requestAdapter=async options=>{const adapter=await request(options);if(!adapter)return adapter;const create=adapter.requestDevice.bind(adapter);adapter.requestDevice=async options=>{const device=await create(options),destroy=device.destroy.bind(device);device.destroy=()=>{destroyed++;destroy();};entered();await gate;return device;};return adapter;};
    const errors:any[]=[];const renderer=f.createRenderer({canvas,initialMode:'dark',initialQuality:'high',onError:(error:any)=>errors.push(error)});await started;
    const cleanup=f.BrowserRenderer.dispose(renderer);let finished=false;cleanup.then(()=>finished=true);await Promise.resolve();await Promise.resolve();const waited=!finished;
    release();await cleanup;await f.BrowserRenderer.ready(renderer);navigator.gpu.requestAdapter=request;
    return {destroyed,contexts,waited,errors:errors.length,status:f.BrowserRenderer.status(renderer)};
  });expect(report.destroyed).toBe(1);expect(report.contexts).toBe(0);expect(report.waited).toBe(true);expect(report.errors).toBe(0);expect(report.status.disposed).toBe(true);
});

test('visibility and offscreen observers pause the loop, resume dirty rendering and disconnect on disposal',async({page})=>{
  await page.emulateMedia({reducedMotion:'reduce'});await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  await page.evaluate(async()=>{
    const f=Reflect.get(window,'shaderFixture'),canvas=document.createElement('canvas');canvas.style.cssText='position:absolute;left:0;top:0;width:64px;height:64px';document.body.append(canvas);
    const errors:string[]=[];const renderer=f.createRenderer({canvas,initialMode:'dark',initialQuality:'high',onError:(error:any)=>errors.push(error.message)});
    Reflect.set(window,'activityFixture',{canvas,renderer,errors});await f.BrowserRenderer.ready(renderer);
  });
  await expect.poll(()=>page.evaluate(()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'activityFixture');return f.BrowserRenderer.status(s.renderer).submittedFrames;})).toBeGreaterThan(2);
  await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:true});document.dispatchEvent(new Event('visibilitychange'));});
  const hidden=await page.evaluate(()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'activityFixture');return f.BrowserRenderer.status(s.renderer);});expect(hidden.running).toBe(false);
  await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:false});document.dispatchEvent(new Event('visibilitychange'));});
  await expect.poll(()=>page.evaluate(()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'activityFixture');return f.BrowserRenderer.status(s.renderer).submittedFrames;})).toBeGreaterThan(hidden.submittedFrames);
  await page.evaluate(()=>{const s=Reflect.get(window,'activityFixture');s.canvas.style.top='3000px';});
  await expect.poll(()=>page.evaluate(()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'activityFixture');return f.BrowserRenderer.status(s.renderer).running;})).toBe(false);
  await page.evaluate(()=>{const s=Reflect.get(window,'activityFixture');s.canvas.style.top='0';});
  await expect.poll(()=>page.evaluate(()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'activityFixture');return f.BrowserRenderer.status(s.renderer).running;})).toBe(true);
  const report=await page.evaluate(async()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'activityFixture');await f.BrowserRenderer.dispose(s.renderer);const before=f.BrowserRenderer.status(s.renderer).submittedFrames;window.dispatchEvent(new Event('resize'));document.dispatchEvent(new Event('visibilitychange'));window.dispatchEvent(new PointerEvent('pointermove',{isPrimary:true,clientX:20,clientY:20}));await new Promise(resolve=>requestAnimationFrame(resolve));return {errors:s.errors,before,after:f.BrowserRenderer.status(s.renderer).submittedFrames,status:f.BrowserRenderer.status(s.renderer)};});
  expect(report.errors).toEqual([]);expect(report.before).toBe(report.after);expect(report.status.disposed).toBe(true);expect(report.status.running).toBe(false);
});

test('unmount during GPU preparation defers device destruction and releases every allocation once',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const f=Reflect.get(window,'shaderFixture'),canvas=document.createElement('canvas');canvas.style.cssText='position:absolute;left:0;top:0;width:64px;height:64px';document.body.append(canvas);
    const made:Array<{label:string,destroyed:number}>=[];let destroyed=0,lateSubmissions=0,unconfigured=0;
    let release!:()=>void,entered!:()=>void;const gate=new Promise<void>(resolve=>release=resolve),started=new Promise<void>(resolve=>entered=resolve);
    const request=navigator.gpu.requestAdapter.bind(navigator.gpu);
    navigator.gpu.requestAdapter=async options=>{const adapter=await request(options);if(!adapter)return adapter;const create=adapter.requestDevice.bind(adapter);adapter.requestDevice=async options=>{
      const device=await create(options),destroy=device.destroy.bind(device);device.destroy=()=>{destroyed++;destroy();};
      const submit=device.queue.submit.bind(device.queue);device.queue.submit=commands=>{if(destroyed)lateSubmissions++;return submit(commands);};
      for(const method of ['createTexture','createBuffer'] as const){const original=device[method].bind(device);Reflect.set(device,method,(descriptor:any)=>{
        const resource=original(descriptor),record={label:descriptor.label??'',destroyed:0};made.push(record);const drop=resource.destroy.bind(resource);resource.destroy=()=>{record.destroyed++;drop();};return resource;
      });}
      const compile=device.createRenderPipelineAsync.bind(device);device.createRenderPipelineAsync=descriptor=>{if(descriptor.label!=='light.present')return compile(descriptor);entered();return gate.then(()=>compile(descriptor));};return device;
    };return adapter;};
    const context=canvas.getContext('webgpu')!,unconfigure=context.unconfigure.bind(context);context.unconfigure=()=>{unconfigured++;unconfigure();};
    const errors:string[]=[];const renderer=f.createRenderer({canvas,initialMode:'light',initialQuality:'high',onError:(error:any)=>errors.push(String(error))});await started;
    const cleanup=f.BrowserRenderer.dispose(renderer),sameCleanup=cleanup===f.BrowserRenderer.dispose(renderer);let finished=false;cleanup.then(()=>finished=true);
    await Promise.resolve();await Promise.resolve();const waited=!finished&&destroyed===0;
    release();await cleanup;await f.BrowserRenderer.ready(renderer);navigator.gpu.requestAdapter=request;
    return {waited,sameCleanup,destroyed,lateSubmissions,unconfigured,errors,count:made.length,leaks:made.filter(record=>record.destroyed!==1),status:f.BrowserRenderer.status(renderer)};
  });expect(report.waited).toBe(true);expect(report.sameCleanup).toBe(true);expect(report.destroyed).toBe(1);expect(report.unconfigured).toBe(1);
  expect(report.lateSubmissions).toBe(0);expect(report.errors).toEqual([]);expect(report.count).toBeGreaterThan(20);expect(report.leaks).toEqual([]);expect(report.status.disposed).toBe(true);
});
