import {expect,test} from "bun:test";
import {existsSync,mkdirSync,mkdtempSync,readFileSync,readdirSync,rmSync,writeFileSync} from "node:fs";
import {tmpdir} from "node:os";
import {dirname,join,resolve} from "node:path";

const root=resolve(import.meta.dir,"..");
const compiler=join(root,"rust-js");

function git(cwd:string,args:string[]) {
  return Bun.spawnSync(["git",...args],{cwd,timeout:10_000,stdout:"pipe",stderr:"pipe"});
}

test("the tracked compiler stack applies cleanly to its published gitlink",()=>{
  const index=git(root,["ls-files","--stage","--","rust-js"]);
  expect(index.exitCode).toBe(0);
  const [mode,pin]=index.stdout.toString().trim().split(/\s+/);
  expect(mode).toBe("160000");
  expect(git(compiler,["rev-parse","HEAD"]).stdout.toString().trim()).toBe(pin);

  const directory=join(root,"patches/rust-js");
  const files=readdirSync(directory).filter(file=>file.endsWith(".patch")).sort();
  expect(files.length).toBeGreaterThan(0);
  const paths=new Set(files.flatMap(file=>[...readFileSync(join(directory,file),"utf8").matchAll(/^\+\+\+ b\/(.+)$/gm)].map(match=>match[1])));
  const edited=new Set<string>();
  for(const args of [["diff","--name-only","-z","HEAD"],["ls-files","--others","--exclude-standard","-z"]]) {
    const result=git(compiler,args);
    expect(result.exitCode).toBe(0);
    for(const path of result.stdout.toString().split("\0").filter(Boolean))edited.add(path);
  }
  expect([...edited].filter(path=>!paths.has(path)),"compiler edits missing from the tracked patch stack").toEqual([]);
  const probe=mkdtempSync(join(tmpdir(),"vgpu-clean-patches-"));
  try {
    for(const path of paths) {
      const original=git(compiler,["show",pin+":"+path]);
      if(original.exitCode===0) {
        const output=join(probe,path);
        mkdirSync(dirname(output),{recursive:true});
        writeFileSync(output,original.stdout);
      }
    }
    expect(git(probe,["init","--quiet"]).exitCode).toBe(0);
    for(const file of files) {
      const applied=git(probe,["apply",join(directory,file)]);
      expect(applied.exitCode,applied.stderr.toString()).toBe(0);
    }
    for(const path of paths) {
      expect(existsSync(join(probe,path))).toBe(true);
      expect(readFileSync(join(probe,path),"utf8")).toBe(readFileSync(join(compiler,path),"utf8"));
    }
  } finally {
    rmSync(probe,{recursive:true,force:true});
  }
});
