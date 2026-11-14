// Test mounting only. Widget behavior is compiled Rust or unchanged reference TS.
import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import {CiRunOutput} from '../../../crates/web/src/ci_run_output.jsx';
import {EveVideo,formatTime} from '../../../crates/web/src/eve_video.jsx';
import {DeferredImage} from '../../../crates/web/src/deferred_image.jsx';
import {OneShaderEverySurfaceSection} from '../../../crates/web/src/one_shader_every_surface.jsx';
import './hero.css';
const original=new URLSearchParams(location.search).has('reference');
const ref=original?await import('./reference-outputs.jsx'):null;
const Ci=ref?.CiRunOutput??CiRunOutput,Video=ref?.EveVideo??EveVideo,Image=ref?.DeferredImage??DeferredImage;
const Section=ref?.OneShaderEverySurfaceSection??OneShaderEverySurfaceSection;
const fullSection=new URLSearchParams(location.search).has("section");
const root=createRoot(document.getElementById('root'));
Object.assign(window,{unmountOutputs:()=>root.unmount(),outputFormatTime:ref?.formatTime??formatTime});
root.render(<StrictMode>{fullSection?<main className="mx-auto max-w-6xl px-6 py-16 lg:px-12"><Section/></main>:<main style={{padding:24,maxWidth:360}}><div data-output-ci><Ci/></div><div style={{height:1200}}/><div data-output-video><Video/></div><div data-output-image style={{height:200,position:'relative'}}><Image src={original?"/examples/eve/eve-dev-home.png":"./examples/eve/eve-dev-home.png"} alt="Deferred Eve homepage" fill sizes="(max-width: 640px) 100vw, 34vw" className="object-cover object-top" loading="lazy" fetchPriority="low"/></div></main>}</StrictMode>);
