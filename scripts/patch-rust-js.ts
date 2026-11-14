// Apply the retained upstream fix to the published submodule pin. Refuse an
// unknown compiler tree rather than guessing whether part of a fix is present.
import {resolve} from 'node:path';
const cwd=resolve(import.meta.dir,'../rust-js');
const patch=resolve(import.meta.dir,'../patches/rust-js/0001-battery-status.patch');
function git(args:string[]) {return Bun.spawnSync(['git',...args],{cwd,stdout:'pipe',stderr:'pipe',timeout:10_000});}
const check=git(['apply','--check',patch]);
if(check.exitCode===0) {
 const applied=git(['apply',patch]);
 if(applied.exitCode!==0) throw new Error(applied.stderr.toString());
} else if(git(['apply','--reverse','--check',patch]).exitCode!==0) {
 throw new Error('Compiler tree does not match the retained Battery Status patch.\n'+check.stderr.toString());
}
