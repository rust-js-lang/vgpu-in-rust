// The pinned gitlink plus tracked patches fully specify the compiler used here.
import {resolve} from 'node:path';
const root=resolve(import.meta.dir,'..');
const patches=resolve(root,'patches/rust-js');
for(const path of [...new Bun.Glob('*.patch').scanSync(patches)].sort()){
 const patch=resolve(patches,path),args=['git','-C',resolve(root,'rust-js'),'apply'];
 const run=(flags:string[])=>Bun.spawnSync([...args,...flags,patch],{timeout:10000,stdout:'pipe',stderr:'pipe'});
 if(run(['--reverse','--check']).exitCode===0)continue;
 const check=run(['--check']);
 if(check.exitCode!==0)throw Error(`Cannot apply ${path}:\n${check.stderr}`);
 const applied=run([]);
 if(applied.exitCode!==0)throw Error(`Failed applying ${path}:\n${applied.stderr}`);
 console.log(`Applied rust-js patch: ${path}`);
}
