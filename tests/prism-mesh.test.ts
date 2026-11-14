import {expect,test} from "bun:test";
import {mkdirSync,readFileSync} from "node:fs";
import {resolve} from "node:path";
import * as mesh from "../crates/prism/src/prism_mesh.js";

const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??"../vgpu");
const directory=resolve(import.meta.dir,"../node_modules/.cache/vgpu-reference-mesh");
const sourcePath=resolve(upstream,"apps/docs/app/[lang]/(home)/components/prism-background/scene/prism-mesh.ts");
mkdirSync(directory,{recursive:true});
await Bun.write(resolve(directory,"entry.ts"),`export {prismMeshData,prismWireframeIndices,prismPlanes} from ${JSON.stringify(sourcePath)};`);
const built=await Bun.build({
  entrypoints:[resolve(directory,"entry.ts")],
  outdir:resolve(directory,"out"),
  target:"bun",
  plugins:[{name:"pure-reference-only",setup(builder){
    builder.onLoad({filter:/\/scene\/prism-mesh\.ts$/},({path})=>{
      const source=readFileSync(path,"utf8");
      const gpuImport='import { geometry } from "vgpu";';
      expect(source.split(gpuImport).length).toBe(2);
      // Only GPU upload calls use this import. The entry exports pure functions;
      // their bodies remain byte-for-byte upstream source. Bun removes unused
      // upload functions. No mock renderer or production code is involved.
      return {contents:source.replace(gpuImport,""),loader:"ts"};
    });
  }}],
});
if(!built.success)throw new AggregateError(built.logs,"Cannot build the upstream pure mesh reference");
const reference=await import(built.outputs[0].path);
const {PRISM_TRIANGLE}=await import(resolve(upstream,"apps/docs/app/[lang]/(home)/components/prism-background/types.ts"));

test("rounded mesh vertices, normals, indices and wireframe match upstream",()=>{
  for(const triangle of [undefined,{a:[0,1],b:[-1,-1],c:[1,-1]},{a:[0.2,0.6],b:[-0.4,-0.2],c:[0.8,-0.3]}]) {
    const shape=triangle??PRISM_TRIANGLE;
    for(const [back,front] of [[0.015,0.315],[-0.5,0.5],[0,0.001],[0.1,0.1],[1,0]]) {
      const actual=mesh.prismMeshData(shape,back,front);
      const expected=reference.prismMeshData(shape,back,front);
      expect(actual.vertices.length).toBe(expected.vertices.length);
      expect(actual.vertices).toEqual(Array.from(expected.vertices));
      expect(actual.indices).toEqual(Array.from(expected.indices));
      expect(mesh.prismWireframeIndices(actual.indices)).toEqual(Array.from(reference.prismWireframeIndices(expected.indices)));
      const planes=mesh.prismPlanes(shape,back,front);
      const expectedPlanes=reference.prismPlanes(shape,back,front);
      for(let i=0;i<planes.length;i++)for(let j=0;j<4;j++)expect(planes[i][j]).toBeCloseTo(expectedPlanes[i][j],12);
    }
  }
});
