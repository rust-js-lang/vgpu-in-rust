import {expect,test} from '@playwright/test';

test('Rust exterior/internal spectral light, environment and two-interface MSAA glass match upstream pixels',async({page})=>{
  await page.goto('/shaders.html');
  await expect.poll(()=>page.evaluate(()=>Boolean(Reflect.get(window,'shaderFixture')))).toBe(true);
  const report=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('A real WebGPU adapter is required');
    const device=await adapter.requestDevice(),fixture=Reflect.get(window,'shaderFixture');
    const reference=await fixture.referenceEnvironment(),gpu=await reference.initFromDevice(device);
    function value(result:any){if(result.TAG==='Err')throw Error(String(result._0));return result._0;}
    function source(suffix:string){return Object.entries(fixture.sources).find(([name])=>name.endsWith(suffix))![1];}
    const backShader=source('/shared/glass/glass-back.wgsl'),frontShader=source('/shared/glass/glass.wgsl'),copyShader=source('/shared/presentation/copy-linear.wgsl'),lightShader=source('/dark/passes/light/light.wgsl');
    const environment=value(fixture.createEnvironmentTexture(device,'glass-studio',false));
    const sampler=fixture.createEnvironmentSampler(device);
    const sceneSampler=device.createSampler({minFilter:'linear',magFilter:'linear',addressModeU:'clamp-to-edge',addressModeV:'clamp-to-edge'});
    const geometry=fixture.createPrismGeometry(device,'glass-prism',false);
    const background=fixture.createRenderTarget(device,[128,128],'rgba16float',1,'glass-background');
    const scene=fixture.createRenderTarget(device,[128,128],'rgba16float',4,'glass-scene');
    const expectedBackground=reference.target(gpu,{size:[128,128],format:'rgba16float'});
    const expectedScene=reference.target(gpu,{size:[128,128],format:'rgba16float',msaa:true});
    const expectedEnvironment=reference.createEnvironmentTexture(gpu,'expected-studio',false);
    const expectedGeometry=reference.prismGeometry(gpu,'expected-prism');
    const resources:any[]=[];let lightBuffer:GPUBuffer|undefined,expectedLightBuffer:any;
    async function pixels(texture:GPUTexture){
      const buffer=device.createBuffer({size:128*128*8,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
      try{
        const encoder=device.createCommandEncoder();encoder.copyTextureToBuffer({texture},{buffer,bytesPerRow:1024},[128,128]);
        device.queue.submit([encoder.finish()]);await buffer.mapAsync(GPUMapMode.READ);
        const data=new Uint16Array(buffer.getMappedRange()).slice();buffer.unmap();return data;
      }finally{buffer.destroy();}
    }
    try{
      device.pushErrorScope('validation');
      value(await fixture.EnvironmentTexture.prepare(environment,device,sampler));
      await reference.prepareEnvironmentTexture(gpu,expectedEnvironment,reference.createEnvironmentSampler(gpu));
      const controls=fixture.defaultPrismControls(),view=fixture.cameraView(1,0.35,-0.2,1.25,48);
      const params=fixture.glassUniforms(view,controls,'dark',false);
      const layout=fixture.lightMeshLayout(128,24),wall=fixture.wallExtent(1,1.25,48,{scale:1,offset:[0,0]});
      const lamp=fixture.lampForIncidence(60,controls.beamWidth,0.5);
      const options={light:lamp,dispersion:controls.spectralDispersion,edgeFalloff:controls.lightFade.edgeFalloff,wallHalfExtent:wall,samples:128,beamSlices:24};
      const mesh=fixture.buildLightMesh(options),expectedMesh=reference.buildLightMesh(options);
      lightBuffer=fixture.createVertexBuffer(device,mesh.vertices,'spectral-vertices');
      expectedLightBuffer=gpu.device.createBuffer({size:expectedMesh.vertices.byteLength,usage:['vertex','copy_dst']});
      expectedLightBuffer.write(expectedMesh.vertices);
      const sceneParams=fixture.sceneUniforms(view,wall,lamp,controls,layout,1);
      const light=await fixture.lightPipeline(device,lightShader,'rgba16float','spectral-light');
      const lightValues=value(fixture.prepareShaderBindings(device,lightShader,[{name:'scene',value:sceneParams}],'spectral-light'));
      resources.push(lightValues);
      const lightGroup=value(fixture.ShaderBindings.bind(lightValues,device,light,[],'spectral-light'));
      const back=await fixture.glassPipeline(device,backShader,'rgba16float','Back',1,'glass-back');
      const front=await fixture.glassPipeline(device,frontShader,'rgba16float','Front',4,'glass-front');
      const copy=await fixture.fullscreenPipeline(device,fixture.fullscreenVertex(device),fixture.shaderModule(device,copyShader,'glass-copy'),'rgba16float',4,'glass-copy');
      const backValues=value(fixture.prepareShaderBindings(device,backShader,[{name:'params',value:params}],'glass-back'));
      const frontValues=value(fixture.prepareShaderBindings(device,frontShader,[{name:'params',value:params}],'glass-front'));
      const copyValues=value(fixture.prepareShaderBindings(device,copyShader,[],'glass-copy'));
      resources.push(backValues,frontValues,copyValues);
      const shared=[{name:'studioEnvironment',resource:environment.texture.createView()},{name:'debugEnvironment',resource:environment.texture.createView()},{name:'environmentSampler',resource:sampler}];
      const backGroup=value(fixture.ShaderBindings.bind(backValues,device,back,shared,'glass-back'));
      const frontGroup=value(fixture.ShaderBindings.bind(frontValues,device,front,[{name:'sceneTexture',resource:background.view},{name:'sceneSampler',resource:sceneSampler},...shared],'glass-front'));
      const copyGroup=value(fixture.ShaderBindings.bind(copyValues,device,copy,[{name:'sceneTexture',resource:background.view}],'glass-copy'));
      const encoder=device.createCommandEncoder();
      const a=encoder.beginRenderPass({colorAttachments:[fixture.RenderTarget.colorAttachment(background,false,[0,0,0,1])]});
      fixture.encodeLight(a,light,lightGroup,lightBuffer,0,layout.whiteVertices);
      fixture.encodeLight(a,light,lightGroup,lightBuffer,layout.outgoingFirstVertex,layout.outgoingVertices);
      fixture.encodePrism(a,back,backGroup,geometry);
      fixture.encodeLight(a,light,lightGroup,lightBuffer,layout.internalFirstVertex,layout.internalVertices);a.end();
      const b=encoder.beginRenderPass({colorAttachments:[fixture.RenderTarget.colorAttachment(scene,false,[0,0,0,1])]});
      fixture.encodeFullscreen(b,copy,copyGroup);fixture.encodePrism(b,front,frontGroup,geometry);b.end();
      device.queue.submit([encoder.finish()]);

      const expectedBack=reference.draw(gpu,{shader:backShader,geometry:expectedGeometry,cull:'front',depth:false,blend:'premultiplied'});
      const expectedFront=reference.draw(gpu,{shader:frontShader,geometry:expectedGeometry,cull:'back',depth:false});
      const expectedCopy=reference.effect(gpu,copyShader);
      const expectedLight=reference.draw(gpu,{shader:lightShader,geometry:{vertexBuffers:[expectedLightBuffer.gpu],
        vertexBufferLayouts:[{arrayStride:12,attributes:[{shaderLocation:0,offset:0,format:'float32x2'},{shaderLocation:3,offset:8,format:'float32'}]}],
        vertexCount:expectedMesh.vertexCount},blend:'additive',cull:'none',depth:false});
      expectedLight.set({scene:sceneParams});
      const common={params,studioEnvironment:expectedEnvironment.texture,debugEnvironment:expectedEnvironment.texture,environmentSampler:sampler};
      expectedBack.set(common);expectedFront.set({...common,sceneTexture:expectedBackground,sceneSampler});expectedCopy.set({sceneTexture:expectedBackground});
      await Promise.all([expectedBack.compile(expectedBackground),expectedFront.compile(expectedScene),expectedCopy.compile(expectedScene),expectedLight.compile(expectedBackground)]);
      reference.frame(gpu,(current:any)=>{
        current.pass({target:expectedBackground,clear:[0,0,0,1]},(pass:any)=>{
          pass.draw(expectedLight,{firstVertex:0,vertices:layout.whiteVertices});
          pass.draw(expectedLight,{firstVertex:layout.outgoingFirstVertex,vertices:layout.outgoingVertices});
          pass.draw(expectedBack);
          pass.draw(expectedLight,{firstVertex:layout.internalFirstVertex,vertices:layout.internalVertices});
        });
        current.pass({target:expectedScene,clear:[0,0,0,1]},(pass:any)=>{pass.draw(expectedCopy);pass.draw(expectedFront);});
      });
      const actual=await pixels(scene.texture),expected=await pixels(expectedScene.color.gpu);
      let mismatches=0,lit=0,nonfinite=0;
      for(let index=0;index<actual.length;index++){
        if(actual[index]!==expected[index])mismatches++;
        if(index%4!==3&&actual[index]!==0)lit++;
        if((actual[index]&0x7c00)===0x7c00)nonfinite++;
      }
      return {mismatches,lit,nonfinite,error:(await device.popErrorScope())?.message??null};
    }finally{
      for(const resource of resources)fixture.ShaderBindings.destroy(resource);
      fixture.RenderTarget.destroy(background);fixture.RenderTarget.destroy(scene);fixture.PrismBuffers.destroy(geometry);
      fixture.EnvironmentTexture.destroy(environment);reference.destroyEnvironmentTexture(expectedEnvironment);
      lightBuffer?.destroy();expectedLightBuffer?.destroy();expectedGeometry.destroy();expectedBackground.destroy?.();expectedScene.destroy?.();gpu.dispose();device.destroy();
    }
  });
  expect(report.error).toBeNull();expect(report.mismatches).toBe(0);expect(report.nonfinite).toBe(0);expect(report.lit).toBeGreaterThan(100);
});
