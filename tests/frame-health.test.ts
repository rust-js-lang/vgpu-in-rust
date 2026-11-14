import {expect, test} from 'bun:test';
import {resolve} from 'node:path';
import {FrameHealthMonitor} from '../crates/prism/src/frame_health.js';
const root=resolve(process.env.VGPU_UPSTREAM_DIR ?? '../vgpu');
const revision=Bun.spawnSync(["git","rev-parse","HEAD"],{cwd:root,timeout:10_000});
expect(revision.exitCode).toBe(0);
expect(revision.stdout.toString().trim()).toBe("773351f8c96d1cbec28155c0e36ae3efd1cba135");
const {createPrismFrameHealthMonitor}=await import(root+'/apps/docs/app/[lang]/(home)/components/prism-background/performance/frame-health.ts');

function equalStatus(actual:any, expected:any) {
  expect(actual.downgrade).toBe(expected.downgrade);
  for(const key of ['estimatedRefreshFps','targetFps','thresholdFps','activeWindowMs']) expect(actual[key]).toBeCloseTo(expected[key],10);
  // Rust Option is represented as undefined at this boundary.
  if(expected.observedFps===undefined) expect(actual.observedFps).toBeUndefined();
  else expect(actual.observedFps).toBeCloseTo(expected.observedFps,10);
}

test('frame health preserves every source status through refresh rates, caps and workload switches',()=>{
  for(const fps of [24,30,59.94,60,75,90,120,144,240]) for(const mobile of [false,true]) for(const workload of ['interactive','dust']) for(const stride of [1,2,3,4]) {
    const port=FrameHealthMonitor.new(), source=createPrismFrameHealthMonitor();
    for(let i=0;i<Math.ceil(fps*3);i++) {
      const sample={deltaMs:1000/fps,active:true,rendered:i%stride===0,mobile,workload};
      equalStatus(FrameHealthMonitor.record(port,sample),source.record(sample));
    }
    FrameHealthMonitor.reset(port); source.reset();
    const sample={deltaMs:16,active:false,rendered:false,mobile,workload};
    equalStatus(FrameHealthMonitor.record(port,sample),source.record(sample));
  }
});

test('frame health matches resets, inactive gaps, invalid deltas and deterministic jitter',()=>{
  const port=FrameHealthMonitor.new(), source=createPrismFrameHealthMonitor();
  let seed=42;
  for(let i=0;i<8000;i++) {
    seed=(Math.imul(seed,1664525)+1013904223)>>>0;
    const deltaMs=i%71===0?[NaN,Infinity,-Infinity,0,-1,250,251,500][Math.floor(i/71)%8]:4+(seed%4700)/100;
    const sample={deltaMs,active:i%97!==0,rendered:seed%5!==0,mobile:i%601>400,workload:i%401>300?'dust':'interactive'};
    equalStatus(FrameHealthMonitor.record(port,sample),source.record(sample));
    if(i%887===0) {FrameHealthMonitor.reset(port);source.reset();}
  }
});
