import {defineConfig} from 'vite';
import tailwindcss from '@tailwindcss/vite';
import {wgslVitePlugin} from '@vgpu/wgsl/loader-vite';
import {resolve} from 'node:path';

const root=resolve(import.meta.dirname,'../..');
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??resolve(root,'../vgpu'),'apps/docs');
export default defineConfig({
  root:resolve(import.meta.dirname,'fixture'),
  base:'./',
  plugins:[tailwindcss(),wgslVitePlugin()],
  resolve:{alias:[{find:/^@vgpu\/wgsl$/,replacement:Bun.resolveSync('@vgpu/wgsl',root)},{find:/^@vgpu\/wgsl\/reflect-source$/,replacement:Bun.resolveSync('@vgpu/wgsl/reflect-source',root)},...Object.entries({'@reference':upstream,'@':upstream,'@reference-api':resolve(upstream,'../../packages/vgpu-api/src'),'vgpu/scene':resolve(import.meta.dirname,'fixture/reference-scene.ts'),'vgpu':resolve(import.meta.dirname,'fixture/reference-vgpu.ts'),'@vgpu/core':resolve(upstream,'../../packages/core/src/index.ts')}).map(([find,replacement])=>({find,replacement}))],dedupe:['react','react-dom','lucide-react']},
  oxc:{jsx:{runtime:'automatic'}},
  build:{outDir:resolve(root,'node_modules/.cache/vgpu-browser'),emptyOutDir:true,rollupOptions:{input:{shaders:resolve(import.meta.dirname,'fixture/shaders.html'),port:resolve(import.meta.dirname,'fixture/index.html'),reference:resolve(import.meta.dirname,'fixture/reference.html')}}},
});
