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
  resolve:{alias:{'@reference':upstream,'@':upstream},dedupe:['react','react-dom','lucide-react']},
  oxc:{jsx:{runtime:'automatic'}},
  build:{outDir:resolve(root,'node_modules/.cache/vgpu-browser'),emptyOutDir:true,rollupOptions:{input:{shaders:resolve(import.meta.dirname,'fixture/shaders.html'),port:resolve(import.meta.dirname,'fixture/index.html'),reference:resolve(import.meta.dirname,'fixture/reference.html')}}},
});
