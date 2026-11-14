import {expect,test} from '@playwright/test';

for(const debug of [false,true])test('Rust runtime retains uploads, lazy edges and one preparation promise; debug='+debug,async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async(debug)=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture');device.pushErrorScope('validation');
    const buffers:Array<{label:string,destroyed:number}>=[],textures:Array<{label:string,destroyed:number}>=[];
    for(const [method,records]of [['createBuffer',buffers],['createTexture',textures]] as const){
      const original=device[method].bind(device);Reflect.set(device,method,(descriptor:any)=>{
        const resource=original(descriptor),record={label:descriptor.label??'',destroyed:0};records.push(record);
        const destroy=resource.destroy.bind(resource);resource.destroy=()=>{record.destroyed++;destroy();};return resource;
      });
    }
    let uploads=0;const write=device.queue.writeBuffer.bind(device.queue);device.queue.writeBuffer=(...args)=>{uploads++;return write(...args);};
    const runtime=f.createPrismRuntime(device,[128,128],'retained',f.lightMeshLayout(64,12),debug);
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    try{
      const initial={buffer:runtime.lightBuffer,vertices:runtime.lightVertices,scratch:runtime.lightVertexScratch,uploads,buffers:buffers.length};
      f.PrismRuntime.setLampAim(runtime,device,0.5,0.5);f.PrismRuntime.resize(runtime,device,[128,128]);f.PrismRuntime.setOrbit(runtime,0.2,-0.1);
      const noUploads=uploads===initial.uploads;
      f.PrismRuntime.setLampAim(runtime,device,0.3,0.7);
      const retainedUpload=runtime.lightBuffer===initial.buffer&&runtime.lightVertices===initial.vertices&&runtime.lightVertexScratch===initial.scratch&&uploads===initial.uploads+1;
      f.PrismRuntime.setLightMeshLayout(runtime,device,f.lightMeshLayout(128,24));
      const layoutStorage=runtime.lightBuffer===initial.buffer&&runtime.lightVertices!==initial.vertices&&runtime.lightVertexScratch===initial.scratch&&buffers.length===initial.buffers;
      const reservedBytes=runtime.lightBuffer.size;
      const before=uploads,controls=structuredClone(runtime.state.controls);controls.postprocess.bloomRadius=2;
      f.PrismRuntime.setControls(runtime,device,controls);const postprocessNoUpload=uploads===before;
      const lazy=runtime.prismWireframe==null,edges=f.PrismRuntime.ensureWireframe(runtime,device),edgeBuffers=buffers.length;
      const cachedEdges=f.PrismRuntime.ensureWireframe(runtime,device)===edges&&buffers.length===edgeBuffers;
      const pending=value(f.PrismRuntime.prepareEnvironment(runtime,device)),samePending=pending===value(f.PrismRuntime.prepareEnvironment(runtime,device));
      await pending;
      const after={buffers:buffers.length,textures:textures.length,uploads};
      const sameFulfilled=pending===value(f.PrismRuntime.prepareEnvironment(runtime,device));await pending;
      const cachedPreparation=after.buffers===buffers.length&&after.textures===textures.length&&after.uploads===uploads;
      const prepared=runtime.studioEnvironment.prepared&&(debug?runtime.debugEnvironment.prepared:runtime.debugEnvironment==null);
      f.PrismRuntime.destroy(runtime);
      return {noUploads,retainedUpload,layoutStorage,reservedBytes,postprocessNoUpload,lazy,cachedEdges,samePending,sameFulfilled,cachedPreparation,prepared,
        buffers,textures,error:(await device.popErrorScope())?.message??null,cleared:runtime.environmentReady==null&&runtime.studioEnvironment==null&&runtime.debugEnvironment==null&&runtime.prismWireframe==null};
    }finally{device.destroy();}
  },debug);
  expect(report.error).toBeNull();
  for(const field of ['noUploads','retainedUpload','layoutStorage','postprocessNoUpload','lazy','cachedEdges','samePending','sameFulfilled','cachedPreparation','prepared','cleared'])expect(report[field],field).toBe(true);
  expect(report.reservedBytes).toBeGreaterThan(1_000_000);
  for(const record of [...report.buffers,...report.textures])expect(record.destroyed,record.label).toBe(1);
});

test('runtime waits for the other GPU bake before propagating a failure and caches the rejected promise',async({page})=>{
  await page.goto('/shaders.html');await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('Real WebGPU required');
    const device=await adapter.requestDevice(),f=Reflect.get(window,'shaderFixture');
    const compile=device.createRenderPipelineAsync.bind(device);
    let release:()=>void=()=>{};const gate=new Promise<void>(resolve=>{release=resolve;});
    device.createRenderPipelineAsync=async descriptor=>{
      if(descriptor.label?.includes('environment-studio.bake'))throw Error('injected studio failure');
      if(descriptor.label?.includes('environment-debug'))await gate;
      return compile(descriptor);
    };
    const runtime=f.createPrismRuntime(device,[64,64],'failure',f.lightMeshLayout(8,3),true);
    const first=f.PrismRuntime.prepareEnvironment(runtime,device);if(first.TAG==='Err')throw Error(first._0);
    let finished=false,message='';const done=first._0.then(()=>{finished=true;},(error:Error)=>{finished=true;message=error.message;});
    await new Promise(resolve=>setTimeout(resolve,20));const waited=!finished;release();await done;
    const second=f.PrismRuntime.prepareEnvironment(runtime,device);
    const debugCompleted=runtime.debugEnvironment.prepared;
    f.PrismRuntime.destroy(runtime);device.destroy();return {waited,message,debugCompleted,cached:second.TAG==='Ok'&&second._0===first._0};
  });
  expect(report).toEqual({waited:true,message:'injected studio failure',debugCompleted:true,cached:true});
});
