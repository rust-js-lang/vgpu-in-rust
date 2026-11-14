import {expect,test} from '@playwright/test';
import {resolve} from 'node:path';
import {optimizeImage} from 'next/dist/server/image-optimizer.js';
// Native image encoding already uses half the cores. Keep this file in order
// so parallel workers do not oversubscribe codecs beside the real GPU suite.
test.describe.configure({mode:'default'});
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
const referenceImageCache=new Map<string,Promise<Buffer>>();
async function referenceImages(page: import('@playwright/test').Page) {
 await page.route('**/_next/image?*',async route=>{
  const url=new URL(route.request().url());
  const source=url.searchParams.get('url')!;
  expect(['/examples/eve/eve-dev-home.png','/examples/eve/eve-billboard.png','/examples/eve/eve-billboard-8k.png']).toContain(source);
  const key=url.search;
  let encoded=referenceImageCache.get(key);
  if(!encoded){
   encoded=(async()=>{const buffer=Buffer.from(await Bun.file(resolve(import.meta.dir,'../../public'+source)).bytes());return optimizeImage({buffer,width:Number(url.searchParams.get('w')),quality:Number(url.searchParams.get('q')),contentType:'image/avif',sequentialRead:true});})();
   referenceImageCache.set(key,encoded);
  }
  await route.fulfill({body:await encoded,contentType:'image/avif'});
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
   await image.evaluate((image:HTMLImageElement)=>image.decode());


   results.push(await image.evaluate((image:HTMLImageElement)=>{
    const canvas=document.createElement('canvas');canvas.width=128;canvas.height=72;const context=canvas.getContext('2d',{willReadFrequently:true})!;context.drawImage(image,0,0,128,72);
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
  const page=await browser.newPage({reducedMotion:'reduce'});await page.addInitScript(mockObservers);await page.clock.install({time:new Date('2020-01-01T00:00:00Z')});await page.clock.pauseAt(new Date('2020-01-01T00:01:00Z'));
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

for(const width of [390,1440])for(const theme of ['light','dark'])test('complete output section preserves '+theme+' content, styles and media at '+width,async({browser},testInfo)=>{
 const results=[];
 for(const original of [false,true]){
  const page=await browser.newPage({viewport:{width,height:1000},deviceScaleFactor:1,reducedMotion:'reduce'});const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
  await page.addInitScript(theme=>document.addEventListener('DOMContentLoaded',()=>document.documentElement.classList.toggle('dark',theme==='dark'),{once:true}),theme);
  try {
   if(original)await referenceImages(page);
   await page.goto('/outputs.html?section'+(original?'&reference':''));
   await expect(page.getByRole('heading',{name:'One shader. Render everywhere.'})).toBeVisible();
   await expect(page.locator('article')).toHaveCount(4);
   // Scroll through the whole responsive stack so each deferred asset is real.
   for(const article of await page.locator('article').all())await article.scrollIntoViewIfNeeded();
   await expect(page.locator('img')).toHaveCount(3);
   await expect.poll(()=>page.locator('img').evaluateAll(images=>images.every((image:HTMLImageElement)=>image.complete&&image.naturalWidth>0))).toBe(true);
   const video=page.locator('video');await expect.poll(()=>video.evaluate((video:HTMLVideoElement)=>video.readyState>=2)).toBe(true);
   await video.evaluate(async(video:HTMLVideoElement)=>{video.pause();if(video.currentTime!==0){video.currentTime=0;await new Promise(resolve=>video.addEventListener('seeked',resolve,{once:true}));}});
   await page.evaluate(async()=>{scrollTo(0,0);await document.fonts.ready;await Promise.all(document.getAnimations().filter(animation=>animation.effect?.getComputedTiming().iterations!==Infinity).map(animation=>animation.finished.catch(()=>{})));});
   const layout=await page.locator('article').evaluateAll(articles=>articles.map(article=>{const {x,y,width,height}=article.getBoundingClientRect();return {x,y,width,height};}));
   // Independent renders of the unchanged upstream differ at rounded-edge AA
   // pixels (150 channels, max delta 2 in the control). Compare every actual
   // element's exact layout/computed CSS, normalized markup and media bytes;
   // keep full screenshots for visual review without inventing a pixel epsilon.
   const state=await page.locator('main').evaluate(async main=>{
    const elements=[main,...main.querySelectorAll('*')].filter(element=>!element.matches('picture,source'));
    const styles=elements.map(element=>{
     const rect=element.getBoundingClientRect(),computed=element.computedStyleMap();
     return {tag:element.tagName,rect:[rect.x,rect.y,rect.width,rect.height],style:Object.fromEntries([...computed].sort(([a],[b])=>a.localeCompare(b)).map(([name,values])=>[name,values.map(value=>value.toString())]))};
    });
    const media=await Promise.all([...main.querySelectorAll('img')].map(async image=>{
     const response=await fetch(image.currentSrc);if(!response.ok)throw Error('Image fetch failed: '+image.currentSrc);
     return Array.from(new Uint8Array(await response.arrayBuffer()));
    }));
    const videos=[...main.querySelectorAll('video')].map(video=>({src:new URL(video.src).pathname,poster:new URL(video.poster).pathname,muted:video.muted,loop:video.loop,playsInline:video.playsInline,preload:video.preload}));
    const clone=main.cloneNode(true) as HTMLElement;
    for(const picture of clone.querySelectorAll('picture'))picture.replaceWith(...picture.childNodes);
    for(const source of clone.querySelectorAll('source'))source.remove();
    for(const element of [clone,...clone.querySelectorAll('*')]){
     for(const name of ['src','srcset','poster','style'])element.removeAttribute(name);
     const attributes=[...element.attributes].map(attribute=>[attribute.name,attribute.value]).sort(([a],[b])=>a.localeCompare(b));
     for(const attribute of [...element.attributes])element.removeAttribute(attribute.name);
     for(const [name,value] of attributes)element.setAttribute(name,value);
    }
    return {styles,markup:clone.outerHTML,media,videos};
   });
   const image=await page.screenshot({fullPage:true,animations:'disabled'});
   const {media,...semantic}=state;results.push({layout,semantic,media:media.map(bytes=>Buffer.from(bytes))});
   await testInfo.attach((original?'reference':'rust')+'-state-'+theme+'-'+width,{body:JSON.stringify(semantic),contentType:'application/json'});
   await testInfo.attach((original?'reference':'rust')+'-'+theme+'-'+width,{body:image,contentType:'image/png'});
   if(!original&&theme==='dark')await Bun.write(resolve(import.meta.dir,'../../docs/screenshots/output-cards-'+(width===390?'mobile':'desktop')+'-checkpoint.png'),image);
   expect(errors).toEqual([]);await page.evaluate(()=>Reflect.get(window,'unmountOutputs')());
  }finally{await page.close();}
 }
 expect(results[0].layout).toEqual(results[1].layout);
 expect(JSON.stringify(results[0].semantic)===JSON.stringify(results[1].semantic),"Exact content, computed styles and geometry must match; inspect the attached state snapshots.").toBe(true);
 expect(results[0].media.map((bytes,index)=>bytes.equals(results[1].media[index])),"Every selected image must match the native optimizer byte-for-byte.").toEqual([true,true,true]);
});
