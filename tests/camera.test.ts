import {expect,test} from 'bun:test';
import {mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import * as camera from '../crates/prism/src/camera.js';
const root=resolve(import.meta.dir,'..');
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu');
const source=resolve(upstream,'apps/docs/app/[lang]/(home)/components/prism-background/scene/camera.ts');
const output=resolve(root,'node_modules/.cache/vgpu-reference-camera');
mkdirSync(output,{recursive:true});
// Select the three original scene exports used by this module, avoiding the
// unrelated GPU geometry/instance barrels. Implementations remain unchanged.
const built=await Bun.build({entrypoints:[source],outdir:output,target:'browser',format:'esm',plugins:[{
  name:'reference-camera-dependencies',setup(build){
    build.onResolve({filter:/^vgpu\/scene$/},()=>({path:'scene',namespace:'original-scene'}));
    build.onLoad({filter:/.*/,namespace:'original-scene'},()=>({loader:'ts',contents:
      'export {group} from '+JSON.stringify(resolve(upstream,'packages/vgpu-api/src/scene/nodes.ts'))+';\n'+
      'export {perspective,viewMatrices} from '+JSON.stringify(resolve(upstream,'packages/vgpu-api/src/scene/camera-state.ts'))+';'}));
    build.onResolve({filter:/^@vgpu\/core$/},()=>({path:resolve(upstream,'packages/core/src/errors.ts')}));
  },
}]});
if(!built.success)throw new AggregateError(built.logs,'Cannot build original camera reference');
const original=await import(built.outputs.find(asset=>asset.path.endsWith('/camera.js'))!.path);

function close(actual:number[],expected:ArrayLike<number>){
  expect(actual.length).toBe(expected.length);
  for(let i=0;i<actual.length;i++)expect(actual[i]).toBeCloseTo(expected[i],12);
}

test('orbital camera matrices and stored Float32 values match the original scene',()=>{
  for(const aspect of [0.25,0.5625,1,16/9,3.5])
    for(const orbit of [[0,0],[-1,-1],[1,1],[-0.7,0.2],[0.13,-0.92],[4,-8]])
      for(const distance of [0.8,1.25,4]){
        const actual=camera.cameraView(aspect,...orbit,distance,48);
        const expected=original.cameraView(aspect,...orbit,distance,48);
        for(const key of ['position','forward','right','up'])close(actual[key],expected[key]);
        for(const key of ['position','quaternion','target','up','projection','view','viewProjection']){
          expect(actual.camera[key],key+' '+JSON.stringify({aspect,orbit,distance})).toEqual(Array.from(expected.camera[key]));
        }
        expect(actual.camera.lens).toEqual(expected.camera.lens);
        expect(actual.camera.aspect).toBe(expected.camera.aspect);
        expect(actual.viewProjection).toBe(actual.camera.viewProjection);
        expect(Array.from(actual.viewProjection)).toEqual(Array.from(expected.viewProjection));
      }
});

test('wall coverage and environment rotation match source across framing controls',()=>{
  for(const aspect of [0.25,0.5625,1,16/9,3.5])
    for(const distance of [0.8,1.25,4])
      for(const fov of [20,48,85]){
        expect(camera.wallHalfHeight(aspect,distance,fov)).toBeCloseTo(original.wallHalfHeight(aspect,distance,fov),12);
        for(const x of [-1,0,1])for(const y of [-1,0,1]){
          expect(camera.wallCoverage(aspect,x,y,distance,fov)).toBeCloseTo(original.wallCoverage(aspect,x,y,distance,fov),12);
        }
      }
  for(const degrees of [[0,0,0],[20,35,-15],[90,0,180],[-36,212,17.5]]){
    expect(camera.rotationMatrix(degrees)).toEqual(Array.from(original.rotationMatrix(degrees)));
  }
});
