import {expect,test} from '@playwright/test';

test('disabled hero keeps the complete setup UI without requesting a GPU',async({page})=>{
 await page.addInitScript(()=>{Reflect.set(window,'adapterCalls',0);const request=navigator.gpu.requestAdapter.bind(navigator.gpu);navigator.gpu.requestAdapter=(...args)=>{Reflect.set(window,'adapterCalls',Reflect.get(window,'adapterCalls')+1);return request(...args);};});
 await page.goto('/hero.html?no-canvas');await expect(page.getByRole('heading',{name:'vgpu'})).toBeVisible();
 await expect(page.locator('canvas')).toHaveCount(0);expect(await page.evaluate(()=>Reflect.get(window,'adapterCalls'))).toBe(0);
 await expect(page.getByText('The WebGPU library, designed for agents.')).toBeVisible();
 await page.getByRole('tab',{name:'CLI',exact:true}).click();await expect(page.locator('[data-hero-tabs-copy]')).toHaveAttribute('aria-label','Copy: pnpm add vgpu');
 await expect(page.locator('a[href="https://vgpu.sh/en/docs/get-started"]')).toHaveText('Get started');
 await expect(page.locator('a[href="https://vgpu.sh/en/examples"]')).toHaveText('Explore examples');
});

for(const mobile of [false,true])for(const initial of ['dark','light'])test('React hero renders native GPU output, follows the site theme and cleans up; mobile='+mobile+', initial='+initial,async({page})=>{
 await page.setViewportSize(mobile?{width:390,height:844}:{width:1280,height:900});await page.emulateMedia({reducedMotion:'reduce'});
 await page.addInitScript(initial=>{
  const resources:any[]=[],observers:any[]=[];Reflect.set(window,'heroResources',resources);Reflect.set(window,'heroObservers',observers);
  Reflect.set(window,'initialHeroTheme',initial);
  const pendingTextures=new WeakMap<GPUDevice,GPUTexture>();
  const getTexture=GPUCanvasContext.prototype.getCurrentTexture;
  GPUCanvasContext.prototype.getCurrentTexture=function(){const texture=getTexture.call(this),config=this.getConfiguration();if(config)pendingTextures.set(config.device,texture);return texture;};
  const request=GPUAdapter.prototype.requestDevice;
  GPUAdapter.prototype.requestDevice=function(...args){return request.apply(this,args).then(device=>{
   const record={destroyed:0,submitted:0,afterDestroy:0};resources.push(record);
   const destroy=device.destroy.bind(device),submit=device.queue.submit.bind(device.queue);
   device.destroy=()=>{record.destroyed++;destroy();};device.queue.submit=(commands)=>{
    record.submitted++;if(record.destroyed)record.afterDestroy++;submit(commands);
    const texture=pendingTextures.get(device);pendingTextures.delete(device);
    if(!texture)return;
    // Capture the submitted surface while its current texture is still valid.
    // An idle canvas may hand out a new empty swapchain texture on a later rAF.
    const buffer=device.createBuffer({size:12*256*4,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST}),encoder=device.createCommandEncoder();let index=0;
    for(const y of [0.2,0.5,0.8])for(const x of [0.1,0.35,0.65,0.9])encoder.copyTextureToBuffer({texture,origin:[Math.min(texture.width-4,Math.floor(texture.width*x)),Math.min(texture.height-4,Math.floor(texture.height*y))]},{buffer,offset:index++*256*4,bytesPerRow:256},[4,4]);
    submit([encoder.finish()]);
    buffer.mapAsync(GPUMapMode.READ).then(()=>{
     const bytes=new Uint8Array(buffer.getMappedRange());let total=0;
     for(let row=0;row<48;row++)for(let x=0;x<4;x++)for(let c=0;c<3;c++)total+=bytes[row*256+x*4+c];
     buffer.unmap();Reflect.set(window,'heroPixelTotal',total);buffer.destroy();
    },()=>buffer.destroy());
   };return device;
  });};
  const observe=MutationObserver.prototype.observe,disconnect=MutationObserver.prototype.disconnect;
  MutationObserver.prototype.observe=function(target,options){if(target===document.documentElement&&options?.attributeFilter?.includes('class'))observers.push({observer:this,disconnected:0});return observe.call(this,target,options);};
  MutationObserver.prototype.disconnect=function(){for(const record of observers)if(record.observer===this)record.disconnected++;return disconnect.call(this);};
 },initial);
 const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
 await page.goto('/hero.html');await expect(page.getByRole('heading',{name:'vgpu'})).toBeVisible();
 await expect.poll(()=>page.evaluate(()=>Reflect.get(window,'heroResources').reduce((n:number,resource:any)=>n+resource.submitted,0))).toBeGreaterThan(0);
 await expect(page.locator('canvas')).toHaveAttribute('aria-hidden','true');
 const slot=await page.locator('[data-triangle-container]').boundingBox();expect(slot).not.toBeNull();expect(slot!.width).toBeGreaterThan(50);expect(slot!.height).toBeGreaterThan(50);
 async function sample(){return page.evaluate(()=>Reflect.get(window,'heroPixelTotal')??0);}
 let first=await sample();await expect.poll(async()=>first=await sample()).toBeGreaterThan(0);
 if(initial==='dark')await page.screenshot({path:'/private/tmp/vgpu-hero-'+(mobile?'mobile':'desktop')+'.png',fullPage:true});
 await page.evaluate(()=>document.documentElement.classList.toggle('light'));const before=first;
 await expect.poll(async()=>Math.abs(await sample()-before),{timeout:8000}).toBeGreaterThan(5000);
 await page.evaluate(()=>Reflect.get(window,'renderHero')(false));await expect(page.locator('canvas')).toHaveCount(0);
 await expect.poll(()=>page.evaluate(()=>Reflect.get(window,'heroResources').every((record:any)=>record.destroyed===1))).toBe(true);
 const report=await page.evaluate(()=>({resources:Reflect.get(window,'heroResources'),observers:Reflect.get(window,'heroObservers').map((record:any)=>record.disconnected)}));
 expect(report.resources.length).toBeGreaterThan(0);expect(report.resources.every((record:any)=>record.afterDestroy===0)).toBe(true);expect(report.observers.length).toBeGreaterThan(0);expect(report.observers.every((count:number)=>count===1)).toBe(true);expect(errors).toEqual([]);
});

test('unavailable WebGPU leaves the original hero content usable and reports initialization failure',async({page})=>{
 await page.addInitScript(()=>{Object.defineProperty(navigator,'gpu',{configurable:true,value:undefined});});
 const failures:string[]=[],errors:string[]=[];page.on('console',message=>{if(message.type()==='error')failures.push(message.text());});page.on('pageerror',error=>errors.push(error.message));
 await page.goto('/hero.html');await expect(page.getByText('The WebGPU library, designed for agents.')).toBeVisible();await expect.poll(()=>failures.filter(message=>message.includes('Prism background failed to render.')).length).toBe(1);
 await page.getByRole('tab',{name:'MCP',exact:true}).click();await expect(page.locator('[data-hero-tabs-copy]')).toHaveAttribute('aria-label','Copy: npx -y add-mcp https://vgpu.sh/api/mcp -g');expect(errors).toEqual([]);
});
