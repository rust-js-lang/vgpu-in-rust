import {expect,test} from 'bun:test';
import {createRequire} from 'node:module';
import sharp from 'sharp';
import {resolve} from 'node:path';
import {getSharp,optimizeImage} from 'next/dist/server/image-optimizer.js';
import {getImageProps} from 'next/image';
import {imageConfigDefault} from 'next/dist/shared/lib/image-config.js';
const root=resolve(import.meta.dir,'..');
const widths=[...imageConfigDefault.deviceSizes,...imageConfigDefault.imageSizes].sort((a,b)=>a-b);
const manifest=await Bun.file(resolve(root,'assets/generated/static-images.json')).json();
test('static image catalog covers every native responsive width and format for each retained image',async()=>{
 expect(manifest.quality).toBe(imageConfigDefault.qualities![0]);
 expect(manifest.concurrency).toBe(getSharp().concurrency());
 for(const source of Object.values(manifest.images) as any[])for(const width of widths)expect(source.variants[width]).toBeDefined();
 for(const [path,bytes] of Object.entries(manifest.outputs)){expect((await Bun.file(resolve(root,'public',path)).bytes()).length).toBe((bytes as {bytes:number}).bytes);}
 expect(Object.keys(manifest.outputs)).toHaveLength(Object.values(manifest.images).reduce((count:number,image:any)=>count+Object.keys(image.variants).length*manifest.formats.length,0));
});
for(const [index,source] of Object.keys(manifest.images).entries())for(const [format,mime] of [['avif','image/avif'],['webp','image/webp'],['png','image/png']])test('static '+format+' pixels use the original Next encoder: '+source,async()=>{
 const width=imageConfigDefault.deviceSizes[[0,3,7][index]],url=manifest.images[source].variants[width][format];
 const buffer=Buffer.from(await Bun.file(resolve(root,'public',source.slice(2))).bytes());
 const expected=await optimizeImage({buffer,width,quality:manifest.quality,contentType:mime,sequentialRead:true});
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

test('Next and static assets use the same locked native image encoder',async()=>{
 const config=await Bun.file(resolve(root,'package.json')).json();
 expect(config.overrides.sharp).toBe(config.devDependencies.sharp);
 const require=createRequire(Bun.resolveSync('next/dist/server/image-optimizer.js',root));
 expect(require.resolve('sharp')).toBe(createRequire(import.meta.url).resolve('sharp'));
 expect(require('sharp').versions).toEqual(sharp.versions);
 expect(sharp.versions.sharp).toBe(config.devDependencies.sharp);
});

test('retained output media is byte-identical to the pinned upstream assets',async()=>{
 const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??resolve(root,'../vgpu'),'apps/docs/public/examples/eve');
 for(const name of ['eve-billboard.png','eve-billboard-8k.png','eve-dev-home.png','eve-hero-1080p.mp4','hudson-yards-billboard.svg']){
  const actual=Buffer.from(await Bun.file(resolve(root,'public/examples/eve',name)).bytes());
  expect(actual.equals(Buffer.from(await Bun.file(resolve(upstream,name)).bytes())),name).toBe(true);
 }
});
