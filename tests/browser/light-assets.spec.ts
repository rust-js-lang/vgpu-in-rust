import {expect,test} from '@playwright/test';
import {resolve} from 'node:path';
const mask=resolve(import.meta.dirname,'../../assets/prism-light/wall-global-light-mask.webp');

for(const mode of ['mask','no-bitmap','wrong-size','decode-retry','sync-compute','cpu-recovery'])test('light material assets and every mip match upstream: '+mode,async({page})=>{
  await page.route('**/hero/prism-light/wall-global-light-mask.webp',route=>route.fulfill({path:mask,contentType:'image/webp'}));
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async(mode)=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture'),ref=await f.referenceEnvironment(),gpu=await ref.initFromDevice(device);
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    const create=globalThis.createImageBitmap,close=ImageBitmap.prototype.close;
    let decodes=0,closes=0,failedOptions=0;
    ImageBitmap.prototype.close=function(){closes++;return close.call(this);};
    if(mode==='no-bitmap')Reflect.set(globalThis,'createImageBitmap',undefined);
    else globalThis.createImageBitmap=async(...args:any[])=>{
      decodes++;
      if(mode==='wrong-size'){const canvas=new OffscreenCanvas(1,1);canvas.getContext('2d')!.fillRect(0,0,1,1);return canvas.transferToImageBitmap();}
      if(mode==='decode-retry'&&args[1]){failedOptions++;throw Error('unsupported image options');}
      return Reflect.apply(create,globalThis,args);
    };
    const compute=device.createComputePipelineAsync.bind(device);
    if(mode==='sync-compute')Reflect.set(device,'createComputePipelineAsync',undefined);
    if(mode==='cpu-recovery')device.createComputePipelineAsync=()=>Promise.reject(Error('injected compute rejection'));
    let actual:any,expected:any;
    try{
      device.pushErrorScope('validation');
      const pending=f.loadPreloadedWallMask(),samePending=pending===f.loadPreloadedWallMask();await pending;
      const sameFulfilled=pending===f.loadPreloadedWallMask();
      actual=value(await(mode==='cpu-recovery'?f.loadLightAssetTextures(device):f.bakeLightAssetTextures(device)));
      expected=mode==='cpu-recovery'?await ref.loadLightAssetTextures(gpu):await ref.bakeLightAssetTextures(gpu);
      const bitmapClosed=closes;
      const probe=device.createShaderModule({code:'@group(0) @binding(0) var source:texture_2d<f32>; @fragment fn fs_main(@builtin(position) p:vec4f)->@location(0) vec4f {return textureLoad(source,vec2i(p.xy),0);}'});
      const pipeline=await f.fullscreenPipeline(device,f.fullscreenVertex(device),probe,'rgba8unorm',1,'read-material');
      async function pixels(texture:GPUTexture,level:number){
        const w=Math.max(1,texture.width>>level),h=Math.max(1,texture.height>>level),stride=Math.ceil(w*4/256)*256;
        const target=f.createRenderTarget(device,[w,h],'rgba8unorm',1,'material-probe');
        const buffer=device.createBuffer({size:stride*h,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});
        try{
          const group=device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:texture.createView({baseMipLevel:level,mipLevelCount:1})}]});
          const encoder=device.createCommandEncoder(),pass=encoder.beginRenderPass({colorAttachments:[f.RenderTarget.colorAttachment(target,false,[0,0,0,1])]});
          f.encodeFullscreen(pass,pipeline,group);pass.end();encoder.copyTextureToBuffer({texture:target.texture},{buffer,bytesPerRow:stride},[w,h]);device.queue.submit([encoder.finish()]);
          await buffer.mapAsync(GPUMapMode.READ);const mapped=new Uint8Array(buffer.getMappedRange()),bytes=new Uint8Array(w*h*4);
          for(let row=0;row<h;row++)bytes.set(mapped.subarray(row*stride,row*stride+w*4),row*w*4);
          buffer.unmap();return bytes;
        }finally{buffer.destroy();f.RenderTarget.destroy(target);}
      }
      const levels:Array<{name:string,level:number,different:number,rgb:number}>=[];
      for(const name of ['wallMaterial','wallLighting','causticProfile']){
        const texture=actual[name] as GPUTexture,original=expected[name].gpu as GPUTexture;
        if(texture.mipLevelCount!==original.mipLevelCount)throw Error('Mip count mismatch');
        for(let level=0;level<texture.mipLevelCount;level++){
          const a=await pixels(texture,level),b=await pixels(original,level);let different=0,rgb=0;
          for(let index=0;index<a.length;index++){if(a[index]!==b[index])different++;if(index%4!==3&&a[index]>0)rgb++;}
          levels.push({name,level,different,rgb});
        }
      }
      return {levels,samePending,sameFulfilled,decodes,failedOptions,bitmapClosed,error:(await device.popErrorScope())?.message??null};
    }finally{
      if(actual)f.LightAssetTextures.destroy(actual);if(expected)ref.destroyLightAssetTextures(expected);
      globalThis.createImageBitmap=create;ImageBitmap.prototype.close=close;gpu.dispose();device.destroy();
    }
  },mode);
  expect(report.error).toBeNull();expect(report.samePending).toBe(true);expect(report.sameFulfilled).toBe(true);expect(report.levels).toHaveLength(31);
  for(const level of report.levels){expect(level.different,JSON.stringify(level)).toBe(0);expect(level.rgb,JSON.stringify(level)).toBeGreaterThan(0);}
  expect(report.bitmapClosed).toBe(mode==='no-bitmap'?0:2);expect(report.decodes).toBe(mode==='no-bitmap'?0:mode==='decode-retry'?4:2);expect(report.failedOptions).toBe(mode==='decode-retry'?2:0);
});

test('failed shared mask fetches clear the cached promise and can retry',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const url=await page.evaluate(()=>Reflect.get(window,'shaderFixture').wallMaskUrl);let requests=0;
  await page.route(new URL(url,page.url()).href,route=>++requests===1?route.fulfill({status:503,body:'retry'}):route.fulfill({path:mask,contentType:'image/webp'}));
  const result=await page.evaluate(async()=>{
    const f=Reflect.get(window,'shaderFixture');const failed=f.loadPreloadedWallMask();const same=failed===f.loadPreloadedWallMask();
    let error='';try{await failed;}catch(reason){error=String((reason as Error).message);}
    const retry=f.loadPreloadedWallMask(),newPromise=failed!==retry;const blob=await retry;
    return {same,newPromise,cached:retry===f.loadPreloadedWallMask(),size:blob.size,error};
  });
  expect(requests).toBe(2);expect(result.same).toBe(true);expect(result.newPromise).toBe(true);expect(result.cached).toBe(true);expect(result.size).toBeGreaterThan(0);expect(result.error).toContain('503');
});

for(const failure of ['submission','upload','generation'])test('light material '+failure+' failure releases all successful and partial textures',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async(failure)=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture');
    const records:Array<{label:string,destroyed:number}>=[],create=device.createTexture.bind(device);
    device.createTexture=descriptor=>{const texture=create(descriptor),record={label:descriptor.label??'',destroyed:0};records.push(record);const destroy=texture.destroy.bind(texture);texture.destroy=()=>{record.destroyed++;destroy();};return texture;};
    const submit=device.queue.submit.bind(device.queue),upload=device.queue.writeTexture.bind(device.queue),exp=Math.exp;
    let injected=false;
    if(failure==='submission')device.queue.submit=()=>{throw Error('injected submission failure');};
    if(failure==='upload')device.queue.writeTexture=(destination,...rest)=>{
      if(destination.texture.label.endsWith('wall-lighting'))throw Error('injected upload failure');return upload(destination,...rest);
    };
    if(failure==='generation')Math.exp=value=>{if(!injected){injected=true;throw Error('injected generation failure');}return exp(value);};
    try{
      let result:any;try{result=await(failure==='submission'?f.bakeLightAssetTextures(device):f.loadRecoveryAssetTextures(device));}catch(error){result={uncaught:(error as Error).message};}
      return {result,records,injected};
    }finally{device.queue.submit=submit;device.queue.writeTexture=upload;Math.exp=exp;device.destroy();}
  },failure);
  expect(report.result).toEqual({TAG:'Err',_0:'injected '+failure+' failure'});
  expect(report.records).toHaveLength(failure==='submission'?4:failure==='upload'?3:2);
  for(const record of report.records)expect(record.destroyed,record.label).toBe(1);
});
