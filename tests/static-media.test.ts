import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
import {optimizeImage} from 'next/dist/server/image-optimizer.js';
import {getImageProps} from 'next/image';
import {imageConfigDefault} from 'next/dist/shared/lib/image-config.js';
const root=resolve(import.meta.dir,'..');
const widths=[...imageConfigDefault.deviceSizes,...imageConfigDefault.imageSizes].sort((a,b)=>a-b);
const manifest=await Bun.file(resolve(root,'assets/generated/static-images.json')).json();
test('static image catalog covers every native responsive width and format for each retained image',async()=>{
 expect(manifest.quality).toBe(imageConfigDefault.qualities![0]);
 for(const source of Object.values(manifest.images) as any[])for(const width of widths)expect(source.variants[width]).toBeDefined();
 for(const [path,bytes] of Object.entries(manifest.outputs)){expect((await Bun.file(resolve(root,'public',path)).bytes()).length).toBe((bytes as {bytes:number}).bytes);}
 expect(Object.keys(manifest.outputs)).toHaveLength(Object.values(manifest.images).reduce((count:number,image:any)=>count+Object.keys(image.variants).length*manifest.formats.length,0));
});
for(const [index,source] of Object.keys(manifest.images).entries())for(const [format,mime] of [['avif','image/avif'],['webp','image/webp'],['png','image/png']])test('static '+format+' pixels use the original Next encoder: '+source,async()=>{
 const width=imageConfigDefault.deviceSizes[[0,3,7][index]],url=manifest.images[source].variants[width][format];
 const buffer=Buffer.from(await Bun.file(resolve(root,'public',source.slice(2))).bytes());
 const expected=await optimizeImage({buffer,width,quality:manifest.quality,contentType:mime,sequentialRead:true,concurrency:7});
 expect(await Bun.file(resolve(root,'public',url.slice(2))).bytes()).toEqual(new Uint8Array(expected));
});

test('the catalog contains every width requested by native Next for landing image sizes',()=>{
 const requested=new Set<number>();
 for(const src of Object.keys(manifest.images))for(const sizes of ['(max-width: 640px) 100vw, 34vw','(max-width: 1024px) 100vw, 50vw','(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw',undefined]){
  getImageProps({src,alt:'',fill:true,sizes,loader:({width})=>{requested.add(width);expect(manifest.images[src].variants[width]).toBeDefined();return './variant-'+width+'.png';}});
 }
 // 34vw permits this image-size candidate; deviceSizes alone omits it.
 expect(requested.has(256)).toBe(true);
});
