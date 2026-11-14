import {expect,test} from '@playwright/test';

test('Rust instanced dust matches upstream across lifetimes and reveal with one retained uniform buffer',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture'),ref=await f.referenceEnvironment(),gpu=await ref.initFromDevice(device);
    const shader=Object.entries(f.sources).find(([name])=>name.endsWith('/dust.wgsl'))![1];
    const color=f.createRenderTarget(device,[128,128],'rgba16float',1,'color'),light=f.createRenderTarget(device,[128,128],'rgba16float',1,'light');
    const output=f.createRenderTarget(device,[128,128],'rgba8unorm',1,'output'),expected=ref.target(gpu,{size:[128,128],format:'rgba8unorm'});
    const encoder=device.createCommandEncoder();for(const target of [color,light])encoder.beginRenderPass({colorAttachments:[f.RenderTarget.colorAttachment(target,false,[0.2,0.4,0.8,1])]}).end();device.queue.submit([encoder.finish()]);
    const sampler=device.createSampler({minFilter:'linear',magFilter:'linear'}),view=f.cameraView(1,0.15,-0.2,1.25,48);
    const extent=f.wallExtent(1,1.25,48,{scale:1,offset:[0,0]});
    const initial=f.dustUniforms(view,extent,[128,128],1.25,0,1);
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    const draw=value(await f.createDustDraw(device,initial,color.view,light.view,sampler,'rgba8unorm','dust'));
    const original=ref.draw(gpu,{shader,vertices:6,instances:2200,cull:'none',depth:false,blend:'additive'});
    const buffer=device.createBuffer({size:512*128,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});
    async function pixels(texture:GPUTexture){
      const encoder=device.createCommandEncoder();encoder.copyTextureToBuffer({texture},{buffer,bytesPerRow:512},[128,128]);device.queue.submit([encoder.finish()]);
      await buffer.mapAsync(GPUMapMode.READ);const bytes=new Uint8Array(buffer.getMappedRange()).slice();buffer.unmap();return bytes;
    }
    let allocations=0;const make=device.createBuffer.bind(device);device.createBuffer=(descriptor)=>{allocations++;return make(descriptor);};
    try{
      device.pushErrorScope('validation');
      original.set({params:{viewProjection:view.viewProjection,fieldHalfExtent:extent,outputSize:[128,128],time:0,cameraDistance:1.25,
        lightPlaneZ:ref.PRISM_LIGHT_PLANE_Z,prismA:ref.PRISM_TRIANGLE.a,prismB:ref.PRISM_TRIANGLE.b,prismC:ref.PRISM_TRIANGLE.c,prismFrontZ:ref.PRISM_FRONT_Z,revealProgress:1},
        colorTexture:color.texture,lightTexture:light.texture,lightSampler:sampler});
      await original.compile(expected);let actualAllocations=0;
      const reports:Array<{time:number,reveal:number,mismatches:number,lit:number,checksum:number}>=[];
      for(const [time,reveal]of [[0,1],[3.2,1],[9.7,1],[1000000,1],[3.2,0],[3.2,0.25]]){
        const before=allocations;
        value(f.ShaderBindings.writeValue(draw.values,device.queue,'params',f.dustUniforms(view,extent,[128,128],1.25,time,reveal)));
        const encoder=device.createCommandEncoder(),pass=encoder.beginRenderPass({colorAttachments:[f.RenderTarget.colorAttachment(output,false,[0,0,0,1])]});
        f.encodeDust(draw,pass);pass.end();device.queue.submit([encoder.finish()]);
        actualAllocations+=allocations-before;
        original.set({params:{time,revealProgress:reveal}});
        ref.frame(gpu,(current:any)=>current.pass({target:expected,clear:[0,0,0,1]},(pass:any)=>pass.draw(original,{instances:2200})));
        const actual=await pixels(output.texture),reference=await pixels(expected.color.gpu);let mismatches=0,lit=0,checksum=0;
        for(let index=0;index<actual.length;index++){if(actual[index]!==reference[index])mismatches++;if(index%4!==3){if(actual[index]>0)lit++;checksum=(checksum+actual[index]*(index+1))%1000000007;}}
        reports.push({time,reveal,mismatches,lit,checksum});
      }
      return {reports,allocatedDuringAnimation:actualAllocations,sharedMatrix:initial.viewProjection===view.viewProjection,error:(await device.popErrorScope())?.message??null};
    }finally{
      f.BoundPipeline.destroy(draw);for(const target of [color,light,output])f.RenderTarget.destroy(target);expected.destroy?.();original.destroy?.();buffer.destroy();gpu.dispose();device.destroy();
    }
  });
  expect(report.error).toBeNull();expect(report.sharedMatrix).toBe(true);expect(report.allocatedDuringAnimation).toBe(0);
  for(const frame of report.reports){expect(frame.mismatches,JSON.stringify(frame)).toBe(0);if(frame.reveal===0)expect(frame.lit).toBe(0);else expect(frame.lit).toBeGreaterThan(10);}
  expect(new Set(report.reports.filter(x=>x.reveal===1).map(x=>x.checksum)).size).toBe(4);
});
