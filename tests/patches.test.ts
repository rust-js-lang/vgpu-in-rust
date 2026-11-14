import {expect,test} from 'bun:test';
import {mkdtempSync,mkdirSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,resolve,join} from 'node:path';
test('retained Battery Status fix applies cleanly to the published compiler pin',()=>{
 const root=resolve(import.meta.dir,'..'), compiler=join(root,'rust-js');
 const patch=join(root,'patches/rust-js/0001-battery-status.patch');
 const work=mkdtempSync(join(tmpdir(),'vgpu-battery-patch-'));
 const git=(args:string[],cwd=work)=>Bun.spawnSync(['git',...args],{cwd,timeout:10000,stdout:'pipe',stderr:'pipe'});
 try {
  expect(git(['init','--quiet']).exitCode).toBe(0);
  for(const name of ['webapi/generate.ts','webapi/src/lib.rs','docs/decisions/0281-webapi-covers-typescripts-dom.md','docs/guide/libraries/README.md']) {
   const file=git(['show','HEAD:'+name],compiler);expect(file.exitCode).toBe(0);
   mkdirSync(dirname(join(work,name)),{recursive:true});writeFileSync(join(work,name),file.stdout);
  }
  expect(git(['apply','--check',patch]).exitCode).toBe(0);
  expect(git(['apply',patch]).exitCode).toBe(0);
  expect(git(['apply','--reverse','--check',patch]).exitCode).toBe(0);
  expect(git(['apply','--check',patch]).exitCode).not.toBe(0);
  expect(git(['apply','--reverse',patch]).exitCode).toBe(0);
  expect(git(['apply','--check',patch]).exitCode).toBe(0);
 } finally {rmSync(work,{recursive:true,force:true});}
});
