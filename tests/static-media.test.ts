import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
import {optimizeImage} from 'next/dist/server/image-optimizer.js';
import {imageConfigDefault} from 'next/dist/shared/lib/image-config.js';
const root=resolve(import.meta.dir,'..');
const manifest=await Bun.file(resolve(root,'assets/generated/static-images.json')).json();
test('static image catalog covers every native responsive width and format for each retained image',async()=>{
 expect(manifest.quality).toBe(imageConfigDefault.qualities![0]);
 for(const source of Object.values(manifest.images) as any[])expect(Object.keys(source.variants).map(Number)).toEqual(imageConfigDefault.deviceSizes);
 for(const [path,bytes] of Object.entries(manifest.outputs)){expect((await Bun.file(resolve(root,'public',path)).bytes()).length).toBe((bytes as {bytes:number}).bytes);}
 expect(Object.keys(manifest.outputs)).toHaveLength(72);
});
for(const [index,source] of Object.keys(manifest.images).entries())for(const [format,mime] of [['avif','image/avif'],['webp','image/webp'],['png','image/png']])test('static '+format+' pixels use the original Next encoder: '+source,async()=>{
 const width=imageConfigDefault.deviceSizes[[0,3,7][index]],url=manifest.images[source].variants[width][format];
 const buffer=Buffer.from(await Bun.file(resolve(root,'public',source.slice(2))).bytes());
 const expected=await optimizeImage({buffer,width,quality:manifest.quality,contentType:mime,sequentialRead:true,concurrency:7});
 expect(await Bun.file(resolve(root,'public',url.slice(2))).bytes()).toEqual(new Uint8Array(expected));
});
