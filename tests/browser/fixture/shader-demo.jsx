// Test mounting only: all application behavior is Rust or unchanged upstream.
import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import {ShaderCodeScalesSection} from '../../../crates/web/src/shader_code_scales_section.jsx';
import './hero.css';
const params=new URLSearchParams(location.search),original=params.has('reference');
const reference=original?await import('@reference/app/[lang]/(home)/components/shader-code-scales-section.tsx'):null;
const Section=reference?.ShaderCodeScalesSection??ShaderCodeScalesSection;
const root=createRoot(document.getElementById('root'));
Object.assign(window,{
 unmountShaderDemo:()=>root.unmount(),
 renderDirect:async()=>{
  const {renderShaderPreview}=original?await import('@reference/app/[lang]/(home)/components/shader-code-scales-renderer.ts'):await import('../../../crates/web/src/shader_preview.js');
  const stop=await renderShaderPreview(document.querySelector('canvas'));
  Reflect.set(window,'stopDirect',stop);
 },
});
root.render(params.has('renderer')?<canvas style={{width:128,height:80}}/>:<StrictMode><main className="mx-auto max-w-6xl px-6 py-16 lg:px-12"><Section/></main></StrictMode>);
