import {expect,test} from '@playwright/test';
test('default Auto loads the actual vendor detector and same-origin benchmark assets after a High frame',async({page})=>{
 const benchmarkRequests:string[]=[];page.on('request',request=>{if(request.url().includes('prism-gpu-benchmarks/'))benchmarkRequests.push(request.url());});
 await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
 const result=await page.evaluate(async()=>{
  const f=Reflect.get(window,'shaderFixture'),logs:any[]=[],errors:string[]=[];
  const canvas=document.createElement('canvas');canvas.style.cssText='width:64px;height:64px;position:absolute;top:0;left:0';document.body.append(canvas);
  let renderer:any;
  renderer=f.createRenderer({canvas,initialMode:'dark',initialQuality:'auto',onError:(error:unknown)=>errors.push(String(error)),qualityLogger:(message:string,details:unknown)=>logs.push({message,details,frames:f.BrowserRenderer.status(renderer).submittedFrames})});
  try {
   await f.BrowserRenderer.ready(renderer);
   for(let i=0;i<200;i++){if(logs.some(log=>log.message.includes('GPU detected.')))break;await new Promise(resolve=>setTimeout(resolve,10));}
   return {logs,errors,state:f.BrowserRenderer.getQualityState(renderer),frames:f.BrowserRenderer.status(renderer).submittedFrames};
  } finally {await f.BrowserRenderer.dispose(renderer);canvas.remove();}
 });
 expect(result.errors).toEqual([]);expect(result.frames).toBeGreaterThan(0);
 const detected=result.logs.find(log=>log.message==='[Prism quality] GPU detected.');expect(detected,JSON.stringify(result.logs)).toBeDefined();expect(detected.frames).toBeGreaterThan(0);
 expect(result.logs.filter(log=>log.message.includes('GPU detection unavailable.'))).toEqual([]);
 const manifest=await page.request.get('/prism-gpu-benchmarks/manifest.json');expect(manifest.ok()).toBe(true);expect((await manifest.json()).files).toHaveLength(16);
 for(const url of benchmarkRequests)expect(new URL(url).origin).toBe(new URL(page.url()).origin);
});
