import {expect,test} from '@playwright/test';
for(const scenario of ['gpu-low','battery-low','battery-events','battery-reject','battery-throw','gpu-reject','gpu-throw','late-dispose','runtime','health-reset'])test('Auto advisory controller matches upstream: '+scenario,async({page})=>{
 await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
 const report=await page.evaluate(async scenario=>{
  const fixture=Reflect.get(window,'shaderFixture'),port=await fixture.autoQuality(),ref=await fixture.referenceEnvironment();
  async function run(original:boolean){
   const logs:any[]=[],events:string[]=[],listeners:any[]=[];
   const manager=new EventTarget();Object.assign(manager,{level:scenario==='battery-low'?0.3:0.9,charging:false});
   const add=manager.addEventListener.bind(manager),remove=manager.removeEventListener.bind(manager);
   const handlers=new Map<string,EventListener>();
   manager.addEventListener=(name,handler,options)=>{listeners.push(['add',name]);handlers.set(name,handler as EventListener);return add(name,handler,options);};
   manager.removeEventListener=(name,handler,options)=>{listeners.push(['remove',name,handlers.get(name)===handler]);return remove(name,handler,options);};
   let resolveTier:(result:unknown)=>void=()=>{},resolveBattery:(result:unknown)=>void=()=>{};
   const pendingTier=new Promise(resolve=>{resolveTier=resolve;}),pendingBattery=new Promise(resolve=>{resolveBattery=resolve;});
   const navigator={getBattery:()=>scenario==='late-dispose'?pendingBattery:scenario==='battery-reject'?Promise.reject('battery denied'):scenario==='battery-throw'?(()=>{throw new Error('battery throw');})():Promise.resolve(manager)};
   const loadGpuTier=()=>scenario==='late-dispose'?pendingTier:scenario==='gpu-reject'?Promise.reject('gpu denied'):scenario==='gpu-throw'?(()=>{throw new Error('gpu throw');})():Promise.resolve({type:'BENCHMARK',tier:scenario==='gpu-low'?1:3,isMobile:false,gpu:'test GPU',fps:60,device:'test'});
   const log=(message:string,details:unknown)=>logs.push(JSON.parse(JSON.stringify([message,details])));
   const options={navigator,loadGpuTier,onDowngrade:(reason:string)=>events.push(reason),logger:original?{info:log}:log};
   const controller=original?ref.createPrismAutoQualityController(options):port.createAutoQualityController(options);
   const flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
   if(scenario==='late-dispose')controller.dispose();
   resolveTier({type:'BLOCKLISTED',tier:0});resolveBattery(manager);await flush();
   if(scenario==='battery-events'){
    Reflect.set(manager,'charging',true);Reflect.set(manager,'level',0.1);manager.dispatchEvent(new Event('levelchange'));
    Reflect.set(manager,'charging',false);manager.dispatchEvent(new Event('chargingchange'));manager.dispatchEvent(new Event('levelchange'));
   }
   if(scenario==='runtime'||scenario==='health-reset'){
    const sample={deltaMs:1000/60,active:true,rendered:false,mobile:false,workload:'interactive'};
    if(scenario==='health-reset'){for(let i=0;i<90;i++)controller.recordFrame(sample);controller.resetHealth();}
    for(let i=0;i<119;i++)controller.recordFrame(sample);
    events.push('before-window');controller.recordFrame(sample);controller.recordFrame(sample);
   }
   controller.dispose();controller.dispose();
   const stopped=logs.length;Reflect.set(manager,'level',0.1);manager.dispatchEvent(new Event('levelchange'));
   return {events,logs,listeners,stopped:stopped===logs.length};
  }
  return {actual:await run(false),expected:await run(true)};
 },scenario);
 expect(report.actual).toEqual(report.expected);expect(report.actual.stopped).toBe(true);
 if(scenario==='runtime'||scenario==='health-reset')expect(report.actual.events).toEqual(['before-window','runtime']);
 if(scenario==='late-dispose')expect(report.actual.events).toEqual([]);
});
