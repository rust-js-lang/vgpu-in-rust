import {expect,test} from '@playwright/test';

// Retain whole submitted textures before an idle swapchain changes identity.
function recordPreview() {
 const resources:any[]=[],captures:Promise<any>[]=[],observers:any[]=[];
 Object.assign(window,{previewResources:resources,previewCaptures:captures,previewObservers:observers});
 const current=new WeakMap<GPUDevice,GPUTexture>();
 const texture=GPUCanvasContext.prototype.getCurrentTexture;
 GPUCanvasContext.prototype.getCurrentTexture=function(){const result=texture.call(this);current.set(this.getConfiguration()!.device,result);return result;};
 const request=GPUAdapter.prototype.requestDevice;
 GPUAdapter.prototype.requestDevice=function(...args){return request.apply(this,args).then(device=>{
  const record={destroyed:0,submitted:0,afterDestroy:0,errors:[] as string[]};resources.push(record);
  device.addEventListener('uncapturederror',event=>record.errors.push(event.error.message));
  const destroy=device.destroy.bind(device),submit=device.queue.submit.bind(device.queue);
  device.destroy=()=>{record.destroyed++;destroy();};
  device.queue.submit=(commands)=>{
   record.submitted++;if(record.destroyed)record.afterDestroy++;submit(commands);
   const output=current.get(device);current.delete(device);if(!output)return;
   if(Reflect.get(window,"skipPreviewReadback"))return;
   const {width,height}=output,bytesPerRow=Math.ceil(width*4/256)*256;
   const buffer=device.createBuffer({size:bytesPerRow*height,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
   const encoder=device.createCommandEncoder();encoder.copyTextureToBuffer({texture:output},{buffer,bytesPerRow},[width,height]);submit([encoder.finish()]);
   captures.push(buffer.mapAsync(GPUMapMode.READ).then(()=>{
    const data=new Uint8Array(buffer.getMappedRange()),pixels=[];
    for(let row=0;row<height;row++)pixels.push(...data.slice(row*bytesPerRow,row*bytesPerRow+width*4));
    buffer.unmap();buffer.destroy();return {width,height,pixels};
   }));
  };
  return device;
 });};
 class Observer {
  targets:Element[]=[];disconnected=0;
  constructor(public callback:ResizeObserverCallback){observers.push(this);}
  observe(target:Element){this.targets.push(target);}
  disconnect(){this.disconnected++;this.targets=[];}
 }
 Reflect.set(window,'ResizeObserver',Observer);
}
function controlActivity() {
 const observers:any[]=[];Reflect.set(window,'shaderObservers',observers);
 class Observer {
  targets:Element[]=[];disconnected=0;
  constructor(public callback:IntersectionObserverCallback,public options?:IntersectionObserverInit){observers.push(this);}
  observe(target:Element){this.targets.push(target);}
  disconnect(){this.targets=[];this.disconnected++;}
 }
 Reflect.set(window,'IntersectionObserver',Observer);
 Reflect.set(window,'setShaderNear',(near:boolean)=>{for(const observer of observers)if(observer.targets.length)observer.callback([{isIntersecting:near}],observer);});
 Reflect.set(window,'setShaderVisible',(visible:boolean)=>{Object.defineProperty(document,'visibilityState',{configurable:true,value:visible?'visible':'hidden'});document.dispatchEvent(new Event('visibilitychange'));});
}
const url=(original:boolean,direct=false)=>'/shader-demo.html?'+(direct?'renderer&':'')+(original?'reference':'');

test('live shader pixels, resize coalescing, idle scheduling and cleanup match upstream exactly',async({browser})=>{
 const states=[];
 for(const original of [false,true]){
  const page=await browser.newPage();const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));await page.addInitScript(recordPreview);
  try{
   await page.goto(url(original,true));await expect(page.locator('canvas')).toHaveCount(1);
   await page.evaluate(()=>Reflect.get(window,'renderDirect')());
   const first=await page.evaluate(()=>Promise.all(Reflect.get(window,'previewCaptures')));
   expect(first).toHaveLength(1);expect(first[0].pixels.some((value:number,index:number)=>index%4!==3&&value>10)).toBe(true);
   const count=await page.evaluate(()=>Reflect.get(window,'previewResources')[0].submitted);
   await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
   expect(await page.evaluate(()=>Reflect.get(window,'previewResources')[0].submitted)).toBe(count);
   await page.evaluate(async()=>{
    const canvas=document.querySelector('canvas')!;canvas.style.width='192px';canvas.style.height='96px';
    const observer=Reflect.get(window,'previewObservers')[0];observer.callback([],observer);observer.callback([],observer);
    await new Promise(resolve=>requestAnimationFrame(resolve));
   });
   const frames=await page.evaluate(()=>Promise.all(Reflect.get(window,'previewCaptures')));expect(frames).toHaveLength(2);expect(frames[1]).toMatchObject({width:192,height:96});
   await page.evaluate(()=>{const observer=Reflect.get(window,'previewObservers')[0];observer.callback([],observer);Reflect.get(window,'stopDirect')();});
   await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(resolve)));
   const state=await page.evaluate(()=>({resources:Reflect.get(window,'previewResources'),observers:Reflect.get(window,'previewObservers').map((o:any)=>({disconnected:o.disconnected,targets:o.targets.length}))}));
   expect(state.resources).toEqual([{destroyed:1,submitted:2,afterDestroy:0,errors:[]}]);expect(state.observers).toEqual([{disconnected:1,targets:0}]);expect(errors).toEqual([]);states.push(frames);
  }finally{await page.close();}
 }
 expect(states[0]).toEqual(states[1]);
});

for(const width of [390,1440])for(const theme of ['light','dark'])test('shader editor preserves content, style and keyboard behavior: '+theme+' '+width,async({browser},testInfo)=>{
 const states=[];
 for(const original of [false,true]){
  const page=await browser.newPage({viewport:{width,height:1000},reducedMotion:'reduce'});const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
  await page.addInitScript(theme=>{Object.defineProperty(navigator,'gpu',{configurable:true,value:undefined});document.addEventListener('DOMContentLoaded',()=>document.documentElement.classList.toggle('dark',theme==='dark'),{once:true});},theme);
  try{
   await page.goto(url(original));await expect(page.getByRole('heading',{name:'Shader code that scales.'})).toBeVisible();
   const snapshots=[];
   const capture=()=>page.locator('main').evaluate(async main=>{
    await Promise.all(document.getAnimations().filter(a=>a.effect?.getComputedTiming().iterations!==Infinity).map(a=>a.finished.catch(()=>{})));
    const clone=main.cloneNode(true) as HTMLElement;
    for(const element of [clone,...clone.querySelectorAll('*')]){
     const attrs=[...element.attributes].map(a=>[a.name,a.value]).sort(([a],[b])=>a.localeCompare(b));
     for(const attr of [...element.attributes])element.removeAttribute(attr.name);
     for(const [name,value] of attrs)element.setAttribute(name,value);
    }
    const state={markup:clone.outerHTML,layout:[main,...main.querySelectorAll('*')].map(element=>{
     const r=element.getBoundingClientRect();return {rect:[r.x,r.y,r.width,r.height],style:[...element.computedStyleMap()].sort(([a],[b])=>a.localeCompare(b)).map(([name,values])=>[name,values.map(v=>v.toString())])};
    })};
    const bytes=new TextEncoder().encode(JSON.stringify(state));
    const digest=Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256",bytes))).map(b=>b.toString(16).padStart(2,"0")).join("");
    return {markup:state.markup,digest};
   });
   await page.evaluate(async()=>{await document.fonts.ready;await Promise.all(document.getAnimations().map(a=>a.finished.catch(()=>{})));});
   snapshots.push(await capture());
   await page.getByRole('tab',{name:'main.wgsl',exact:true}).click();await expect(page.locator('#shader-source-panel')).toContainText('shortest = min');
   await page.locator('#shader-source-panel').evaluate(element=>element.scrollTop=100);snapshots.push(await capture());
   const keys=['ArrowRight','ArrowRight','ArrowLeft','Home','End','Escape'];
   for(const key of keys){await page.getByRole('tab',{selected:true}).press(key);await expect(page.getByRole('tab',{selected:true})).toBeFocused();snapshots.push(await capture());}
   await page.getByRole('tab',{name:'main.wgsl',exact:true}).click();await expect.poll(()=>page.locator('#shader-source-panel').evaluate(e=>e.scrollTop)).toBe(0);
   states.push(snapshots);expect(errors).toEqual([]);
   await testInfo.attach((original?'reference':'rust')+'-shader-'+theme+'-'+width,{body:JSON.stringify(snapshots),contentType:'application/json'});
   await testInfo.attach((original?'reference':'rust')+'-shader-'+theme+'-'+width,{body:await page.screenshot({fullPage:true,animations:'disabled'}),contentType:'image/png'});
   await page.evaluate(()=>Reflect.get(window,'unmountShaderDemo')());
  }finally{await page.close();}
 }
 expect(JSON.stringify(states[0])===JSON.stringify(states[1]),'Exact markup, computed CSS, geometry and keyboard states must match; inspect attachments.').toBe(true);
});

for(const width of [390,1440])test('preview starts only near the viewport, follows document visibility and releases every device at '+width,async({browser},testInfo)=>{
 const states=[];
 for(const original of [false,true]){
  const page=await browser.newPage({viewport:{width,height:1000}});await page.addInitScript(recordPreview);await page.addInitScript(controlActivity);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  try{
   await page.goto(url(original));await expect(page.locator('canvas')).toHaveCount(1);expect(await page.evaluate(()=>Reflect.get(window,'previewResources').length)).toBe(0);
   expect(await page.evaluate(()=>Reflect.get(window,'shaderObservers').filter((o:any)=>o.targets.length).map((o:any)=>o.options))).toEqual([{rootMargin:'320px 0px'}]);
   await page.evaluate(()=>Reflect.get(window,'setShaderNear')(true));await expect(page.locator('canvas')).toHaveClass(/opacity-100/);
   await page.evaluate(async()=>{await Promise.all(Reflect.get(window,'previewCaptures'));});
   await page.evaluate(async()=>{await document.fonts.ready;await Promise.all(document.getAnimations().filter(a=>a.effect?.getComputedTiming().iterations!==Infinity).map(a=>a.finished.catch(()=>{})));});
   const image=await page.screenshot({fullPage:true,animations:'disabled'});
   await testInfo.attach((original?'reference':'rust')+'-live-ring-'+width,{body:image,contentType:'image/png'});
   if(!original)await Bun.write(`docs/screenshots/shader-demo-${width===390?"mobile":"desktop"}-checkpoint.png`,image);
   await page.evaluate(()=>Reflect.get(window,'setShaderVisible')(false));await expect.poll(()=>page.evaluate(()=>Reflect.get(window,'previewResources').every((r:any)=>r.destroyed===1))).toBe(true);
   await page.evaluate(()=>Reflect.get(window,'setShaderVisible')(true));await expect.poll(()=>page.evaluate(()=>Reflect.get(window,'previewResources').filter((r:any)=>r.destroyed===0).length)).toBe(1);
   await page.evaluate(async()=>{await Promise.all(Reflect.get(window,'previewCaptures'));});
   await page.evaluate(()=>Reflect.get(window,'setShaderNear')(false));await expect.poll(()=>page.evaluate(()=>Reflect.get(window,'previewResources').every((r:any)=>r.destroyed===1))).toBe(true);
   await page.evaluate(()=>Reflect.get(window,'unmountShaderDemo')());
   const state=await page.evaluate(()=>({resources:Reflect.get(window,'previewResources'),observers:Reflect.get(window,'shaderObservers').map((o:any)=>({disconnected:o.disconnected,targets:o.targets.length}))}));
   expect(state.resources).toHaveLength(2);expect(state.resources.every((r:any)=>r.destroyed===1&&r.afterDestroy===0&&r.errors.length===0)).toBe(true);expect(state.observers.every((o:any)=>o.targets===0&&o.disconnected===1)).toBe(true);expect(errors).toEqual([]);states.push(state);
  }finally{await page.close();}
 }
 expect(states[0]).toEqual(states[1]);
});

test('missing GPU and missing IntersectionObserver retain a usable editor and poster',async({browser})=>{
 for(const original of [false,true]){
  const page=await browser.newPage();const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(()=>{Object.defineProperty(navigator,'gpu',{configurable:true,value:undefined});Reflect.deleteProperty(window,'IntersectionObserver');});
  try{
   await page.goto(url(original));await expect(page.locator('canvas')).toHaveClass(/opacity-0/);
   await page.getByRole('tab',{name:'color.wgsl',exact:true}).click();await expect(page.getByRole('tabpanel')).toContainText('export fn palette');
   await expect(page.getByRole('img',{name:'Rainbow ring rendered from the compiled shader modules'})).toBeVisible();
   expect(errors).toEqual([]);await page.evaluate(()=>Reflect.get(window,'unmountShaderDemo')());
  }finally{await page.close();}
 }
});

test('late GPU initialization after unmount disposes its renderer without reviving the component',async({browser})=>{
 for(const original of [false,true]){
  const page=await browser.newPage();const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(recordPreview);await page.addInitScript(controlActivity);
  await page.addInitScript(()=>{
   Reflect.set(window,'skipPreviewReadback',true);
   const request=GPUAdapter.prototype.requestDevice;
   GPUAdapter.prototype.requestDevice=function(...args){return request.apply(this,args).then(device=>new Promise(resolve=>Reflect.set(window,'releasePreviewDevice',()=>resolve(device))));};
  });
  try{
   await page.goto(url(original));await expect(page.locator('canvas')).toHaveCount(1);
   await page.evaluate(()=>Reflect.get(window,'setShaderNear')(true));
   await expect.poll(()=>page.evaluate(()=>typeof Reflect.get(window,'releasePreviewDevice'))).toBe('function');
   await page.evaluate(()=>Reflect.get(window,'unmountShaderDemo')());await expect(page.locator('canvas')).toHaveCount(0);
   await page.evaluate(()=>Reflect.get(window,'releasePreviewDevice')());
   await expect.poll(()=>page.evaluate(()=>Reflect.get(window,'previewResources').every((r:any)=>r.destroyed===1))).toBe(true);
   const state=await page.evaluate(()=>({resources:Reflect.get(window,'previewResources'),observers:Reflect.get(window,'previewObservers').map((o:any)=>({disconnected:o.disconnected,targets:o.targets.length}))}));
   expect(state.resources).toEqual([{destroyed:1,submitted:1,afterDestroy:0,errors:[]}]);expect(state.observers).toEqual([{disconnected:1,targets:0}]);expect(errors).toEqual([]);
  }finally{await page.close();}
 }
});

test('a failed first draw releases the configured surface and device and retains the failure cause',async({browser})=>{
 for(const original of [false,true]){
  const page=await browser.newPage();await page.addInitScript(recordPreview);
  await page.addInitScript(()=>{
   let unconfigured=0;Reflect.set(window,'previewUnconfigured',()=>unconfigured);
   const unconfigure=GPUCanvasContext.prototype.unconfigure;
   GPUCanvasContext.prototype.unconfigure=function(){unconfigured++;unconfigure.call(this);};
   GPUDevice.prototype.createRenderPipeline=function(){throw Error('injected preview pipeline failure');};
  });
  try{
   await page.goto(url(original,true));await expect(page.locator('canvas')).toHaveCount(1);
   const message=await page.evaluate(async()=>{try{await Reflect.get(window,'renderDirect')();return null;}catch(error){return ((error as Error).cause as Error|undefined)?.message??(error as Error).message;}});
   expect(message).toBe('injected preview pipeline failure');
   expect(await page.evaluate(()=>Reflect.get(window,'previewResources').map((r:any)=>r.destroyed))).toEqual([1]);
   expect(await page.evaluate(()=>Reflect.get(window,'previewUnconfigured')())).toBe(1);
  }finally{await page.close();}
 }
});
