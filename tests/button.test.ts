import {expect,test} from 'bun:test';
import {resolve} from 'node:path';
import {createElement} from 'react';
import {renderToStaticMarkup} from 'react-dom/server';
import {Button,buttonVariants} from '../crates/web/src/button.jsx';
const root=resolve(import.meta.dir,'..'),upstream=resolve(process.env.VGPU_UPSTREAM_DIR??'../vgpu');
const built=await Bun.build({entrypoints:[resolve(upstream,'apps/docs/components/ui/button.tsx')],outdir:resolve(root,'node_modules/.cache/vgpu-reference-button'),target:'browser',format:'esm',external:['react','react/jsx-runtime','radix-ui','class-variance-authority','clsx','tailwind-merge'],plugins:[{name:'original-utils',setup(build){build.onResolve({filter:/^@\/lib\/utils$/},()=>({path:resolve(upstream,'apps/docs/lib/utils.ts')}));}}]});
if(!built.success)throw new AggregateError(built.logs,'Cannot build original button');
const original=await import(built.outputs.find(output=>output.path.endsWith('/button.js'))!.path);
test('Rust button matches original CVA classes, attribute forwarding and Radix Slot markup',()=>{
 for(const variant of [undefined,null,'default','destructive','outline','secondary','ghost','link']) for(const size of [undefined,null,'default','sm','lg','icon','icon-sm','icon-lg']) for(const className of [undefined,'','px-2 bg-white text-black hover:bg-white/90','hero-glass-button text-white shadow-none hover:text-white']) {
  const variants={variant,size,className};expect(buttonVariants(variants)).toBe(original.buttonVariants(variants));
  for(const asChild of [false,true]) {
   const props={...variants,asChild,title:'action',disabled:!asChild,'data-example':'kept',children:asChild?createElement('a',{href:'/target',className:'child-class'},'Get started'):'Click'};
   expect(renderToStaticMarkup(createElement(Button,props))).toBe(renderToStaticMarkup(createElement(original.Button,props)));
  }
 }
});
