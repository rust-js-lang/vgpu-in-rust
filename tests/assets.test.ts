import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
const root=resolve(import.meta.dir,'..');
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu');
const original=resolve(upstream,'apps/docs/app/[lang]/(home)/components/prism-background');

test('retained prism shader graph and fullscreen stage match their original sources exactly',async()=>{
  const paths=[...new Bun.Glob('**/*.wgsl').scanSync(original)].sort();
  const retained=[...new Bun.Glob('**/*.wgsl').scanSync(resolve(root,'assets/prism-background'))].sort();
  expect(retained).toEqual(paths);
  expect(paths.length).toBe(44);
  for(const path of paths){
    expect(await Bun.file(resolve(root,'assets/prism-background',path)).text()).toBe(await Bun.file(resolve(original,path)).text());
  }
  const fullscreen=await import(resolve(upstream,'packages/vgpu-api/src/fullscreen-stage.ts'));
  expect(await Bun.file(resolve(root,'assets/vgpu/fullscreen.wgsl')).text()).toBe(fullscreen.FULLSCREEN_VERTEX_SOURCE);
});


test('retained landing theme and application styles match their original sources exactly',async()=>{
 const app=resolve(upstream,'apps/docs/app');
 for(const file of ['hero-theme.css','hero-glass-button.css','hero-solo.css'])expect(await Bun.file(resolve(root,'assets/styles',file)).text()).toBe(await Bun.file(resolve(app,'[lang]/(home)',file)).text());
 for(const file of ['global.css','styles/geistdocs.css','styles/legacy-vgpu-tokens.css'])expect(await Bun.file(resolve(root,'assets/site',file)).text()).toBe(await Bun.file(resolve(app,file)).text());
});

test("retained landing shader preview modules match upstream byte-for-byte",async()=>{
 const source=resolve(upstream,"apps/docs/app/[lang]/(home)/components/shader-code-scales");
 for(const file of ["main.wgsl","color.wgsl"])expect(await Bun.file(resolve(root,"assets/landing/shader-code-scales",file)).bytes()).toEqual(await Bun.file(resolve(source,file)).bytes());
});
