import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
import {bloomKernelWeights,bloomPairedKernel,bloomBlurUniforms,bloomLevelSize,bloomSpread} from '../crates/prism/src/bloom.js';
import {presentationRevealUniforms,heroRevealProgress} from '../crates/prism/src/presentation.js';
const base=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu','apps/docs/app/[lang]/(home)/components/prism-background/pipelines');
const config=await import(base+'/dark/passes/bloom/config.ts');
const pairing=await import(base+'/dark/passes/bloom/pairing.ts');
const uniforms=await import(base+'/dark/passes/bloom/uniforms.ts');
const presentation=await import(base+'/shared/presentation/index.ts');

test('every padded kernel and bilinear pair retains upstream energy and fractional offsets',()=>{
  for(const taps of [-1,0,1,1.9,6,10,14,18,24,40,NaN,Infinity])for(const capacity of [0,1,12,24]){
    const actual=bloomKernelWeights(taps,capacity),expected=config.bloomKernelWeights(taps,capacity);
    expect(actual).toEqual(expected);
    for(const slots of [0,1,4,12]){
      const paired=bloomPairedKernel(actual,taps,slots);
      try{
        const result=pairing.bloomPairedKernel(expected,taps,slots);
        expect(paired.TAG).toBe('Ok');expect(paired._0).toEqual(result);
      }catch(error){
        expect(paired.TAG).toBe('Err');expect(paired._0).toBe((error as Error).message);
      }
    }
  }
});

test('all pyramid levels retain the original raw/paired WGSL parameter layouts',()=>{
  for(let level=0;level<4;level++)for(const horizontal of [true,false])for(const size of [[1,1],[51,16],[960,540]]){
    expect(bloomBlurUniforms(level,horizontal,size)).toEqual(uniforms.bloomBlurUniforms(level,horizontal?'horizontal':'vertical',size));
  }
  for(const radius of [-1,0.25,0.5,1,3,10,NaN,Infinity])for(const [low,high] of [[0.25,3],[1,1],[NaN,3]])
    expect(bloomSpread(radius,low,high)).toBe(config.bloomSpread(radius,low,high));
  expect(bloomLevelSize([101,31],0)).toEqual([51,16]);
  expect(bloomLevelSize([1,1],9)).toEqual([1,1]);
});

test('display-space theme reveal and independent intro timing retain upstream values',()=>{
  for(const mode of ['dark','light'])for(const progress of [-1,0,0.25,1,2,NaN])
    expect(presentationRevealUniforms(mode,progress)).toEqual(presentation.presentationRevealUniforms(mode,progress));
  for(const elapsed of [-1,0,0.1,0.5,1,1.5,2.5,3,NaN]){
    const actual=heroRevealProgress(elapsed),expected=presentation.heroRevealProgress(elapsed);
    for(const key of ['opacity','beamWidth']){
      if(Number.isNaN(expected[key]))expect(actual[key]).toBeNaN();
      else expect(actual[key]).toBeCloseTo(expected[key],14);
    }
  }
});
