import {expect,test} from '@playwright/test';
import {resolve} from 'node:path';
import {optimizeImage} from 'next/dist/server/image-optimizer.js';
const mockObservers=()=>{
 const records:any[]=[];Reflect.set(window,'outputObservers',records);
 class Observer {
  callback:IntersectionObserverCallback;options:IntersectionObserverInit|undefined;targets:Element[]=[];disconnected=0;
  constructor(callback:IntersectionObserverCallback,options?:IntersectionObserverInit){this.callback=callback;this.options=options;records.push(this);}
  observe(target:Element){this.targets.push(target);}
  disconnect(){this.disconnected++;this.targets=[];}
 }
 Reflect.set(window,'IntersectionObserver',Observer);
 Reflect.set(window,'setOutputVisibility',(visible:boolean,kind='all')=>{for(const observer of [...records])if(observer.targets.length&&(kind!=='ci'||observer.options?.threshold===0.35))observer.callback([{isIntersecting:visible}],observer);});
};
test('CI timing and viewport pause/resume match the original component',async({browser})=>{
 const results=[];
 for(const original of [false,true]){
  const page=await browser.newPage();await page.clock.install({time:new Date('2020-01-01T00:00:00Z')});await page.clock.pauseAt(new Date('2020-01-01T00:01:00Z'));await page.addInitScript(mockObservers);
  try {
   await page.goto('/outputs.html'+(original?'?reference':''));await expect(page.locator('[data-output-ci] p')).toHaveCount(6);
   const snapshots=[];const capture=()=>page.locator('[data-output-ci]').innerHTML();snapshots.push(await capture());
   await page.evaluate(()=>Reflect.get(window,'setOutputVisibility')(true,'ci'));
   for(const time of [649,1,650,650,650,1799,1]){await page.clock.runFor(time);snapshots.push(await capture());}
   await page.evaluate(()=>Reflect.get(window,'setOutputVisibility')(false,'ci'));const paused=await capture();await page.clock.runFor(5000);expect(await capture()).toBe(paused);
   await page.evaluate(()=>Reflect.get(window,'setOutputVisibility')(true,'ci'));await page.clock.runFor(650);snapshots.push(await capture());
   await page.evaluate(()=>Reflect.get(window,'unmountOutputs')());await page.clock.runFor(5000);
   expect(await page.evaluate(()=>Reflect.get(window,'outputObservers').every((observer:any)=>observer.targets.length===0))).toBe(true);
   results.push(snapshots);
  }finally{await page.close();}
 }
 expect(results[0]).toEqual(results[1]);
});
test('deferred video metadata, progress and time formatting match the original',async({browser})=>{
 const results=[];
 for(const original of [false,true]){
  const page=await browser.newPage();await page.addInitScript(mockObservers);
  try {
   await page.goto('/outputs.html'+(original?'?reference':''));await expect(page.locator('video')).toHaveCount(1);await expect(page.locator('video')).not.toHaveAttribute('src');await expect(page.locator('[data-output-image] img')).toHaveCount(0);
   const result=await page.evaluate(async()=>{
    const video=document.querySelector('video')!;const values=[];
    for(const [current,duration] of [[0,0],[61.9,125],[130,125],[-1,125],[NaN,125],[Infinity,Infinity]]){
     Object.defineProperty(video,'currentTime',{configurable:true,value:current});Object.defineProperty(video,'duration',{configurable:true,value:duration});video.dispatchEvent(new Event('timeupdate',{bubbles:true}));await new Promise(resolve=>setTimeout(resolve,0));
     values.push(document.querySelector('[data-output-video]')!.innerHTML);
    }
    const format=Reflect.get(window,'outputFormatTime'),times=[NaN,Infinity,-Infinity,-1,-0,0,0.99,59.99,60,61.9,3600,1e21,1e308].map(value=>format(value));
    return {values,times};
   });results.push(result);
  }finally{await page.close();}
 }
 expect(results[0]).toEqual(results[1]);
});

// The original page uses Next's server optimizer. This test-only endpoint uses
// that same encoder; the port serves build-time derivatives on static hosting.
async function referenceImages(page: import('@playwright/test').Page) {
 await page.route('**/_next/image?*',async route=>{
  const url=new URL(route.request().url());
  expect(url.searchParams.get('url')).toBe('/examples/eve/eve-dev-home.png');
  const buffer=Buffer.from(await Bun.file(resolve(import.meta.dir,'../../public/examples/eve/eve-dev-home.png')).bytes());
  const body=await optimizeImage({buffer,width:Number(url.searchParams.get('w')),quality:Number(url.searchParams.get('q')),contentType:'image/avif',sequentialRead:true});
  await route.fulfill({body,contentType:'image/avif'});
 });
}
for(const viewport of [{width:390,height:844},{width:1440,height:900}])test('deferred image activates and matches native optimized pixels at '+viewport.width,async({browser})=>{
 const results=[];
 for(const original of [false,true]){
  const page=await browser.newPage({viewport,deviceScaleFactor:2});const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));const requests:string[]=[];page.on('request',request=>requests.push(request.url()));
  try {
   if(original)await referenceImages(page);
   await page.goto('/outputs.html'+(original?'?reference':''));
   await expect(page.locator('[data-output-image]')).toHaveCount(1);
   await expect(page.locator('[data-output-image] img')).toHaveCount(0);
   expect(requests.some(url=>url.includes('/examples/eve/')||url.includes('/_next/image'))).toBe(false);
   await page.locator('[data-output-image]').scrollIntoViewIfNeeded();
   const image=page.locator('[data-output-image] img');
   await expect(image).toBeVisible();
   await expect.poll(()=>image.evaluate((image:HTMLImageElement)=>image.complete&&image.naturalWidth>0)).toBe(true);
   results.push(await image.evaluate((image:HTMLImageElement)=>{
    const canvas=document.createElement('canvas');canvas.width=128;canvas.height=72;const context=canvas.getContext('2d')!;context.drawImage(image,0,0,128,72);
    return {pixels:Array.from(context.getImageData(0,0,128,72).data),attributes:['alt','class','sizes','loading','fetchpriority','style'].map(name=>image.getAttribute(name)),width:image.getBoundingClientRect().width,height:image.getBoundingClientRect().height};
   }));
   expect(errors).toEqual([]);
   await page.evaluate(()=>Reflect.get(window,'unmountOutputs')());
  }finally{await page.close();}
 }
 expect(results[0]).toEqual(results[1]);
});
test('deferred retained video actually plays and reports metadata in both implementations',async({browser})=>{
 const durations=[];
 for(const original of [false,true]){
  const page=await browser.newPage();const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
  try {
   if(original)await referenceImages(page);
   await page.goto('/outputs.html'+(original?'?reference':''));
   const video=page.locator('video');await expect(video).not.toHaveAttribute('src');
   await video.scrollIntoViewIfNeeded();
   await expect.poll(()=>video.evaluate((video:HTMLVideoElement)=>video.currentTime>0&&!video.paused)).toBe(true);
   const state=await video.evaluate((video:HTMLVideoElement)=>({duration:video.duration,muted:video.muted,loop:video.loop,playsInline:video.playsInline,preload:video.preload,error:video.error?.code??null}));
   expect(state.duration).toBeGreaterThan(0);expect(state).toMatchObject({muted:true,loop:true,playsInline:true,preload:'none',error:null});durations.push(state.duration);
   await expect(page.locator('[data-output-video]')).not.toContainText('00:00 / 00:00');expect(errors).toEqual([]);
   await page.evaluate(()=>Reflect.get(window,'unmountOutputs')());
  }finally{await page.close();}
 }
 expect(durations[0]).toBe(durations[1]);
});
test('reduced motion reveals the complete CI output immediately without starting its timer',async({browser})=>{
 const results=[];
 for(const original of [false,true]){
  const page=await browser.newPage({reducedMotion:'reduce'});await page.addInitScript(mockObservers);await page.clock.install();await page.clock.pauseAt(new Date());
  try {
   await page.goto('/outputs.html'+(original?'?reference':''));await expect(page.locator('[data-output-ci] p.opacity-100')).toHaveCount(5);
   const before=await page.locator('[data-output-ci]').innerHTML();await page.clock.runFor(10000);expect(await page.locator('[data-output-ci]').innerHTML()).toBe(before);results.push(before);
   expect(await page.evaluate(()=>Reflect.get(window,'outputObservers').some((observer:any)=>observer.options?.threshold===0.35))).toBe(false);
  }finally{await page.close();}
 }
 expect(results[0]).toBe(results[1]);
});

test('missing IntersectionObserver reveals CI output and activates media without throwing',async({browser})=>{
 const states=[];
 for(const original of [false,true]){
  const page=await browser.newPage();const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
  await page.addInitScript(()=>Reflect.deleteProperty(window,'IntersectionObserver'));
  try {
   if(original)await referenceImages(page);
   await page.goto('/outputs.html'+(original?'?reference':''));
   await expect(page.locator('[data-output-ci] p.opacity-100')).toHaveCount(5);
   await expect(page.locator('video')).toHaveAttribute('src',/examples\/eve\/eve-hero-1080p.mp4$/);
   await expect(page.locator('[data-output-image] img')).toHaveCount(1);
   states.push(await page.evaluate(()=>({ci:document.querySelector('[data-output-ci]')!.innerHTML,video:{src:new URL(document.querySelector('video')!.src).pathname,poster:new URL(document.querySelector('video')!.poster).pathname,autoplay:document.querySelector('video')!.autoplay},image:{alt:document.querySelector('img')!.alt,sizes:document.querySelector('img')!.sizes}})));
   await page.evaluate(()=>Reflect.get(window,'unmountOutputs')());expect(errors).toEqual([]);
  }finally{await page.close();}
 }
 expect(states[0]).toEqual(states[1]);
});
