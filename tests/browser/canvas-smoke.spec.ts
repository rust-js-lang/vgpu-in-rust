import {expect,test} from '@playwright/test';
test('browser canvas renders with native animation scheduling and switches theme',async({page})=>{
  await page.emulateMedia({reducedMotion:'reduce'});await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  await page.evaluate(async()=>{
    const f=Reflect.get(window,'shaderFixture'),canvas=document.createElement('canvas');canvas.style.cssText='position:absolute;left:0;top:0;width:256px;height:256px';document.body.append(canvas);
    const errors:string[]=[];const renderer=f.createRenderer({canvas,initialMode:'dark',initialQuality:'high',onError:(error:any)=>errors.push(String(error))});
    Reflect.set(window,'canvasSmoke',{renderer,canvas,errors});await f.BrowserRenderer.ready(renderer);
  });
  await expect.poll(()=>page.evaluate(()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'canvasSmoke');const status=f.BrowserRenderer.status(s.renderer);if(s.errors.length||status.disposed)throw Error(JSON.stringify({status,errors:s.errors}));return status.submittedFrames;})).toBeGreaterThan(2);
  await page.evaluate(async()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'canvasSmoke');const controls=f.defaultPrismControls();controls.wallColor='#d2ccc2';f.BrowserRenderer.setControls(s.renderer,controls);await f.BrowserRenderer.setMode(s.renderer,'light');});
  const report=await page.evaluate(async()=>{const f=Reflect.get(window,'shaderFixture'),s=Reflect.get(window,'canvasSmoke');const status=f.BrowserRenderer.status(s.renderer);await f.BrowserRenderer.dispose(s.renderer);return {status,errors:s.errors};});
  expect(report.errors).toEqual([]);expect(report.status.disposed).toBe(false);expect(report.status.mode).toBe('light');
});

test('native swapchain survives repeated clear submissions',async({page})=>{
  await page.goto('/shaders.html');const report=await page.evaluate(async()=>{
    const gpu=navigator.gpu,adapter=await gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),canvas=document.createElement('canvas');document.body.append(canvas);canvas.width=256;canvas.height=256;
    const context=canvas.getContext('webgpu')!;context.configure({device,format:gpu.getPreferredCanvasFormat(),alphaMode:'premultiplied',colorSpace:'srgb',usage:GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.TEXTURE_BINDING|GPUTextureUsage.COPY_SRC});
    let lost='';device.lost.then(info=>lost=info.message);device.pushErrorScope('validation');
    for(let frame=0;frame<5;frame++)await new Promise<void>(resolve=>requestAnimationFrame(()=>{
      const encoder=device.createCommandEncoder();const pass=encoder.beginRenderPass({colorAttachments:[{view:context.getCurrentTexture().createView(),clearValue:[0.2,0.3,0.4,1],loadOp:'clear',storeOp:'store'}]});pass.end();device.queue.submit([encoder.finish()]);resolve();
    }));
    await device.queue.onSubmittedWorkDone();const error=(await device.popErrorScope())?.message??null;
    context.unconfigure();device.destroy();return {lost,error,gpuFormat:gpu.getPreferredCanvasFormat(),adapterFeatures:adapter.features.size};
  });expect(report.lost).toBe('');expect(report.error).toBeNull();
});
