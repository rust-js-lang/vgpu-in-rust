import {expect,test} from '@playwright/test';

test.use({deviceScaleFactor:2});
for(const mobile of [false,true])for(const reduced of [false,true])test('native canvas renderer matches original frames; mobile='+mobile+', reduced='+reduced,async({page})=>{
  await page.setViewportSize(mobile?{width:390,height:844}:{width:1280,height:900});
  await page.emulateMedia({reducedMotion:reduced?'reduce':'no-preference'});
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  await page.route('**/hero/prism-light/wall-global-light-mask.webp',async route=>{
    const path=await page.evaluate(()=>Reflect.get(window,'shaderFixture').wallMaskUrl);
    const response=await page.request.get(path);await route.fulfill({body:await response.body(),contentType:'image/webp'});
  });
  const report=await page.evaluate(async({mobile})=>{
    const f=Reflect.get(window,'shaderFixture'),ref=await f.referenceEnvironment();
    let now=0,nextFrame=1;const callbacks=new Map<number,FrameRequestCallback>();
    const raf=window.requestAnimationFrame,caf=window.cancelAnimationFrame,clock=performance.now.bind(performance);
    window.requestAnimationFrame=callback=>{const id=nextFrame++;callbacks.set(id,callback);return id;};window.cancelAnimationFrame=id=>{callbacks.delete(id);};
    Object.defineProperty(performance,'now',{configurable:true,value:()=>now});
    const canvases=[document.createElement('canvas'),document.createElement('canvas')];
    for(const canvas of canvases){canvas.style.cssText='position:absolute;left:0;top:0;width:128px;height:128px';document.body.append(canvas);}
    const framing=document.createElement('div');framing.style.cssText='position:absolute;left:38px;top:10px;width:80px;height:100px';document.body.append(framing);
    const errors:string[]=[],sourceErrors:string[]=[];const controls=f.defaultPrismControls();
    const actual=f.createRenderer({canvas:canvases[0],framingElement:framing,initialMode:'dark',initialQuality:'high',initialControls:controls,onError:(error:any)=>errors.push(String(error))});
    const original=ref.createRenderer({canvas:canvases[1],framingElement:framing,initialMode:'dark',initialQuality:'high',initialControls:controls,onError:(error:any)=>sourceErrors.push(String(error))});
    const qualityStates:any[]=[];
    const comparisons:Array<{stage:string,different:number,nonzero:number,size:number[]}>=[];
    async function image(canvas:HTMLCanvasElement){
      const context=canvas.getContext('webgpu')!,device=context.getConfiguration()!.device;
      const row=Math.ceil(canvas.width*4/256)*256;
      const buffer=device.createBuffer({size:row*canvas.height,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
      try{
        const encoder=device.createCommandEncoder();encoder.copyTextureToBuffer({texture:context.getCurrentTexture()},{buffer,bytesPerRow:row},[canvas.width,canvas.height]);
        device.queue.submit([encoder.finish()]);await buffer.mapAsync(GPUMapMode.READ);
        const result=new Uint8Array(buffer.getMappedRange()).slice();buffer.unmap();return result;
      }finally{buffer.destroy();}
    }
    async function compare(stage:string){
      qualityStates.push({stage,actual:f.BrowserRenderer.getQualityState(actual),expected:original.getQualityState()});
      if(canvases[0].width!==canvases[1].width||canvases[0].height!==canvases[1].height)throw Error('Canvas size mismatch');
      const [a,b]=await Promise.all([image(canvases[0]),image(canvases[1])]);let different=0,nonzero=0;
      for(let i=0;i<a.length;i++){if(a[i]!==b[i])different++;if(i%4!==3&&a[i]>0)nonzero++;}
      comparisons.push({stage,different,nonzero,size:[canvases[0].width,canvases[0].height]});
    }
    async function advance(time:number){now=time;const pending=[...callbacks.values()];callbacks.clear();for(const callback of pending)callback(time);await Promise.resolve();}
    let stage="ready";
    try{
      await Promise.all([f.BrowserRenderer.ready(actual),original.ready]);
      for(const time of [0,16.667,33.334,50,100,250,500,1000,2500,3000]){
        if(time===500)window.dispatchEvent(new PointerEvent('pointermove',{clientX:95,clientY:48,isPrimary:true}));
        stage="dark-"+time;await advance(time);await compare('dark-'+time);
      }
      stage="resize";for(const canvas of canvases){canvas.style.width="160px";canvas.style.height="100px";}framing.style.left="20px";framing.style.top="0px";framing.style.width="140px";framing.style.height="100px";window.dispatchEvent(new Event("resize"));await advance(3016.667);await advance(3033.334);await compare("resized");
      stage="switch-light";const light=f.defaultPrismControls();light.wallColor='#d2ccc2';f.BrowserRenderer.setControls(actual,light);original.setControls(light);
      await Promise.all([f.BrowserRenderer.setMode(actual,'light'),original.setMode('light')]);await advance(3050);await compare('light');
      await Promise.all([f.BrowserRenderer.setQualityPreference(actual,'low'),original.setQualityPreference('low')]);
      await advance(3083.334);await compare('low');
      for(let index=1;index<=120;index++)await advance(3083.334+index*16.667);
      await compare('settled');const before=f.BrowserRenderer.status(actual).submittedFrames;
      for(let index=121;index<=141;index++)await advance(3083.334+index*16.667);
      const idleFrames=f.BrowserRenderer.status(actual).submittedFrames-before;
      return {comparisons,qualityStates,errors,sourceErrors,idleFrames,status:f.BrowserRenderer.status(actual),mobile};
    }catch(error){throw Error(JSON.stringify({stage,errors,sourceErrors,status:f.BrowserRenderer.status(actual),error:String(error)}));}finally{
      await f.BrowserRenderer.dispose(actual);original.dispose();
      canvases.forEach(canvas=>canvas.remove());framing.remove();window.requestAnimationFrame=raf;window.cancelAnimationFrame=caf;
      Object.defineProperty(performance,'now',{configurable:true,value:clock});
    }
  },{mobile});
  for(const state of report.qualityStates)expect(state.actual,state.stage).toEqual(state.expected);
  expect(report.errors).toEqual([]);expect(report.sourceErrors).toEqual([]);
  for(const comparison of report.comparisons){expect(comparison.different,JSON.stringify(comparison)).toBe(0);if(comparison.stage==='settled')expect(comparison.nonzero).toBeGreaterThan(100);}
  expect(report.status.quality).toBe('low');expect(report.status.size).toEqual([160,100]);
  if(mobile)expect(report.idleFrames).toBeGreaterThan(0);else expect(report.idleFrames).toBe(0);
});
