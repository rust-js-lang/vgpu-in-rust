import {defineConfig} from 'vite';
import tailwindcss from '@tailwindcss/vite';
import {wgslVitePlugin} from '@vgpu/wgsl/loader-vite';
import {resolve} from 'node:path';

const root=resolve(import.meta.dirname,'../..');
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??resolve(root,'../vgpu'),'apps/docs');
export default defineConfig({
  root:resolve(import.meta.dirname,'fixture'),
  base:'./',
  publicDir:resolve(root,'public'),
  plugins:[tailwindcss(),wgslVitePlugin(),{name:'reference-video-format-export',transform(code,id){if(id===resolve(upstream,'app/[lang]/(home)/components/eve-video.tsx'))return code+'\nexport {formatTime};\n';}}],
  resolve:{alias:[{find:/^next\/image$/,replacement:Bun.resolveSync('next/dist/esm/shared/lib/image-external.js',root)},{find:/^@pmndrs\/detect-gpu$/,replacement:Bun.resolveSync('@pmndrs/detect-gpu',root)},{find:/^@vgpu\/wgsl$/,replacement:Bun.resolveSync('@vgpu/wgsl',root)},{find:/^@vgpu\/wgsl\/reflect-source$/,replacement:Bun.resolveSync('@vgpu/wgsl/reflect-source',root)},...Object.entries({'@reference':upstream,'@':upstream,'@reference-api':resolve(upstream,'../../packages/vgpu-api/src'),'vgpu/core':resolve(upstream,'../../packages/vgpu-api/src/core.ts'),'vgpu/scene':resolve(import.meta.dirname,'fixture/reference-scene.ts'),'vgpu':resolve(import.meta.dirname,'fixture/reference-vgpu.ts'),'@vgpu/core':resolve(upstream,'../../packages/core/src/index.ts')}).map(([find,replacement])=>({find,replacement}))],dedupe:['react','react-dom','lucide-react','next']},
  oxc:{jsx:{runtime:'automatic'}},
  build:{outDir:resolve(root,'node_modules/.cache/vgpu-browser'),emptyOutDir:true,rollupOptions:{input:{outputs:resolve(import.meta.dirname,'fixture/outputs.html'),hero:resolve(import.meta.dirname,'fixture/hero.html'),shaders:resolve(import.meta.dirname,'fixture/shaders.html'),port:resolve(import.meta.dirname,'fixture/index.html'),reference:resolve(import.meta.dirname,'fixture/reference.html')}}},
});
