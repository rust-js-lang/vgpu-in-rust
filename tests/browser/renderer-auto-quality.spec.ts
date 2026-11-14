import {expect,test} from '@playwright/test';
test.use({deviceScaleFactor:2});
for(const scenario of ['switch-before-load','dispose-before-load','downgrade','failed-downgrade','import-reject','import-throw'])test('live Auto renderer protects deferred work: '+scenario,async({page})=>{
 await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
 const report=await page.evaluate(async scenario=>{
  const f=Reflect.get(window,'shaderFixture');let now=performance.now(),next=1;
  const callbacks=new Map<number,FrameRequestCallback>(),raf=window.requestAnimationFrame,caf=window.cancelAnimationFrame,clock=performance.now.bind(performance);
  window.requestAnimationFrame=callback=>{const id=next++;callbacks.set(id,callback);return id;};window.cancelAnimationFrame=id=>{callbacks.delete(id);};
  Object.defineProperty(performance,'now',{configurable:true,value:()=>now});
  const canvas=document.createElement('canvas');canvas.style.cssText='width:64px;height:64px;position:absolute;top:0;left:0';document.body.append(canvas);
  const states:any[]=[],errors:string[]=[],samples:any[]=[];let loadCalls=0,created=0,disposed=0,resets=0;
  let resolveLoad:(factory:unknown)=>void=()=>{};
  const pending=new Promise(resolve=>{resolveLoad=resolve;});let downgrade:(reason:string)=>void=()=>{};
  const factory=(options:any)=>{created++;downgrade=options.onDowngrade;return {recordFrame:(sample:any)=>samples.push(sample),resetHealth:()=>{resets++;},dispose:()=>{disposed++;}};};
  const renderer=f.createRenderer({canvas,initialMode:'dark',initialQuality:'auto',onError:(error:unknown)=>errors.push(String(error)),loadAutoQuality:()=>{loadCalls++;if(scenario==='import-reject')return Promise.reject(new Error('chunk denied'));if(scenario==='import-throw')throw new Error('loader threw');return pending;},qualityLogger:()=>{}});
  const api=f.BrowserRenderer;
  const flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
  const step=async()=>{now+=1000/60;const batch=[...callbacks.values()];callbacks.clear();for(const callback of batch)callback(now);await flush();};
  const settleTask=()=>new Promise(resolve=>setTimeout(resolve,20));
  try {
   await api.ready(renderer);const before={loads:loadCalls,frames:api.status(renderer).submittedFrames};
   const listener=(state:any)=>states.push(state);
   const unsubscribe=api.subscribeQuality(renderer,listener);const duplicate=api.subscribeQuality(renderer,listener);
   await step();const first={loads:loadCalls,frames:api.status(renderer).submittedFrames};
   await step();await settleTask();const requested=loadCalls;
   if(scenario==='switch-before-load')await api.setQualityPreference(renderer,'high');
   if(scenario==='dispose-before-load')await api.dispose(renderer);
   resolveLoad(factory);await flush();
   if(scenario==='downgrade'||scenario==='failed-downgrade'){
    for(let i=0;i<3;i++)await step();
    const device=canvas.getContext('webgpu')!.getConfiguration()!.device,compile=device.createRenderPipelineAsync.bind(device);
    if(scenario==='failed-downgrade')device.createRenderPipelineAsync=()=>Promise.reject(new Error('low pipeline denied'));
    downgrade('battery');
    for(let i=0;i<200;i++){await settleTask();if(scenario==='downgrade'?api.getQualityState(renderer).effective==='low':errors.length>0)break;}
    device.createRenderPipelineAsync=compile;
   }
   if(scenario.startsWith('import-')){await step();await step();await settleTask();}
   const state=api.getQualityState(renderer),size=[canvas.width,canvas.height];
   const statesBeforeUnsubscribe=states.length;unsubscribe();duplicate();
   if(scenario!=='dispose-before-load')await api.setQualityPreference(renderer,'high');
   const unsubscribeWorked=states.length===statesBeforeUnsubscribe;
   downgrade('runtime');await settleTask();
   const staleIgnored=scenario==='dispose-before-load'||api.getQualityState(renderer).effective==='high';
   await api.dispose(renderer);await step();await settleTask();
   return {before,first,requested,created,disposed,resets,state,size,states,errors,unsubscribeWorked,samples,staleIgnored,loadCalls};
  } finally {await api.dispose(renderer);canvas.remove();window.requestAnimationFrame=raf;window.cancelAnimationFrame=caf;Object.defineProperty(performance,'now',{configurable:true,value:clock});}
 },scenario);
 expect(report.before).toEqual({loads:0,frames:0});expect(report.first.loads).toBe(0);expect(report.first.frames).toBeGreaterThan(0);expect(report.requested).toBe(1);
 expect(report.unsubscribeWorked).toBe(true);expect(report.staleIgnored).toBe(true);
 if(scenario==='switch-before-load'||scenario==='dispose-before-load'||scenario.startsWith('import-')){expect(report.created).toBe(0);expect(report.errors).toEqual([]);if(scenario.startsWith('import-'))expect(report.loadCalls).toBeGreaterThan(1);}
 else {
  expect(report.created).toBe(1);expect(report.disposed).toBe(1);expect(report.samples.length).toBeGreaterThan(0);
  if(scenario==='downgrade'){
   expect(report.state).toEqual({preference:'auto',effective:'low',reason:'battery'});expect(report.size).toEqual([64,64]);expect(report.errors).toEqual([]);
   expect(report.states).toEqual([{preference:'auto',effective:'low',reason:'battery'}]);
  }else{
   expect(report.state).toEqual({preference:'auto',effective:'high',reason:'initial'});expect(report.size).toEqual([128,128]);expect(report.errors).toHaveLength(1);expect(report.errors[0]).toContain('low pipeline denied');
  }
 }
});
