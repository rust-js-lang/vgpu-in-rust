import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
import {batteryRequestsLow,gpuTierRequestsLow} from '../crates/prism/src/quality_signals.js';
const root=resolve(process.env.VGPU_UPSTREAM_DIR ?? '../vgpu');
const revision=Bun.spawnSync(["git","rev-parse","HEAD"],{cwd:root,timeout:10_000});
expect(revision.exitCode).toBe(0);
expect(revision.stdout.toString().trim()).toBe("773351f8c96d1cbec28155c0e36ae3efd1cba135");
const ref=await import(root+'/apps/docs/app/[lang]/(home)/components/prism-background/performance/auto-quality.ts');
test('GPU tier and battery advisory decisions preserve source boundaries',()=>{
 for(const type of ['BLOCKLISTED','BENCHMARK','FALLBACK','WEBGL_UNSUPPORTED'] as const) for(const tier of [-1,0,1,1.00001,2,3,NaN,Infinity,-Infinity]) for(const isMobile of [undefined,false,true]) {
  const signal={type,tier,isMobile};expect(gpuTierRequestsLow(signal)).toBe(ref.gpuTierRequestsLow(signal));
 }
 expect(batteryRequestsLow(undefined)).toBe(ref.batteryRequestsLow(undefined));
 for(const charging of [true,false]) for(const level of [-1,0,0.299999,0.3,0.300001,1,NaN,Infinity,-Infinity]) {
  const signal={charging,level};expect(batteryRequestsLow(signal)).toBe(ref.batteryRequestsLow(signal));
 }
});
