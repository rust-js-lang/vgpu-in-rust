import {expect,test} from '@playwright/test';
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
test('CI timing, viewport pause/resume and reduced motion match the original component',async({browser})=>{
 const results=[];
 for(const original of [false,true]){
  const page=await browser.newPage();await page.clock.install({time:new Date('2020-01-01T00:00:00Z')});await page.clock.pauseAt(new Date('2020-01-01T00:01:00Z'));await page.addInitScript(mockObservers);
  try {
   await page.goto('/outputs.html'+(original?'?reference':''));await expect(page.locator('[data-output-ci] p')).toHaveCount(6);
   const snapshots=[];const capture=()=>page.locator('[data-output-ci]').innerHTML();snapshots.push(await capture());
   await page.evaluate(()=>Reflect.get(window,'setOutputVisibility')(true,'ci'));
   for(const time of [649,1,650,650,650,1799,1]){await page.clock.runFor(time);snapshots.push(await capture());}
   await page.evaluate(()=>Reflect.get(window,'setOutputVisibility')(false,'ci'));const paused=await capture();await page.clock.runFor(5000);expect(await capture()).toBe(paused);
   await page.evaluate(()=>Reflect.get(window,'setOutputVisibility')(true));await page.clock.runFor(650);snapshots.push(await capture());
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
