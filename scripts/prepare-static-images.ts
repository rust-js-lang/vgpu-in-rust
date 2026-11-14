import {createHash} from 'node:crypto';
import {availableParallelism} from 'node:os';
import {resolve} from 'node:path';
import {mkdir} from 'node:fs/promises';
import sharp from 'sharp';
import {imageConfigDefault} from 'next/dist/shared/lib/image-config.js';
// Same geometry and encoders as Next 16.3.3 image-optimizer.ts. Pages serves
// these immutable files; <picture> performs browser format selection.
const root=resolve(import.meta.dir,'..');
const sources=['eve-billboard.png','eve-billboard-8k.png','eve-dev-home.png'];
const formats=[{name:'avif',mime:'image/avif'},{name:'webp',mime:'image/webp'},{name:'png',mime:'image/png'}] as const;
const widths=imageConfigDefault.deviceSizes,quality=imageConfigDefault.qualities![0];
const manifestPath=resolve(root,'assets/generated/static-images.json');
const hash=(bytes:Uint8Array|string)=>createHash('sha256').update(bytes).digest('hex');
const packageConfig=await Bun.file(resolve(root,'package.json')).json();
const inputs=await Promise.all(sources.map(async file=>({file,bytes:await Bun.file(resolve(root,'public/examples/eve',file)).bytes()})));
const fingerprint=hash(JSON.stringify({version:1,code:hash(await Bun.file(import.meta.path).bytes()),next:packageConfig.dependencies.next,sharp:sharp.versions,widths,quality,formats,sources:inputs.map(({file,bytes})=>({file,sha256:hash(bytes)}))}));
let previous:any;
if(await Bun.file(manifestPath).exists())previous=await Bun.file(manifestPath).json();
const valid=previous?.fingerprint===fingerprint&&await Promise.all(Object.entries(previous.outputs??{}).map(async([path,entry])=>{try{const bytes=await Bun.file(resolve(root,'public',path)).bytes();const expected=entry as {bytes:number,sha256:string};return bytes.length===expected.bytes&&hash(bytes)===expected.sha256;}catch{return false;}})).then(values=>values.every(Boolean));
if(valid){console.log('Static images are current.');process.exit(0);}
if(process.argv.includes('--check'))throw Error('Static images are missing or stale. Run bun run assets:prepare.');
const jobs=inputs.flatMap(input=>widths.flatMap(width=>formats.map(format=>({input,width,format}))));
const images:Record<string,{variants:Record<string,Record<string,string>>}>={},outputs:Record<string,{bytes:number,sha256:string}>={};
for(const {file} of inputs)images['./examples/eve/'+file]={variants:Object.fromEntries(widths.map(width=>[String(width),{}]))};
sharp.concurrency(Math.ceil(availableParallelism()/2));
await mkdir(resolve(root,'public/examples/eve'),{recursive:true});
let next=0;
await Promise.all(Array.from({length:2},async()=>{
 while(next<jobs.length){
  const {input,width,format}=jobs[next++];
  const image=sharp(input.bytes,{sequentialRead:true}).rotate().resize(width,undefined,{withoutEnlargement:true});
  if(format.name==='avif')image.avif({quality:Math.max(Math.round(quality*(50/80)),1),effort:3});
  else if(format.name==='webp')image.webp({quality});else image.png({quality});
  const bytes=await image.toBuffer(),path='examples/eve/'+input.file.slice(0,-4)+'.w'+width+'.q'+quality+'.'+format.name;
  await Bun.write(resolve(root,'public',path),bytes);outputs[path]={bytes:bytes.length,sha256:hash(bytes)};
  images['./examples/eve/'+input.file].variants[String(width)][format.name]='./'+path;
 }
}));
const ordered=Object.fromEntries(Object.keys(images).sort().map(key=>[key,{variants:Object.fromEntries(widths.map(width=>[String(width),Object.fromEntries(formats.map(format=>[format.name,images[key].variants[String(width)][format.name]]))]))}]));
await Bun.write(manifestPath,JSON.stringify({fingerprint,quality,formats,fallback:'png',images:ordered,outputs:Object.fromEntries(Object.entries(outputs).sort())},null,2)+'\n');
console.log('Generated '+jobs.length+' responsive images using Next '+packageConfig.dependencies.next+' settings.');
