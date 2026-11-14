// Test mounting only; the hero and its renderer are compiled Rust.
import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import {Hero} from '../../../crates/web/src/hero.jsx';
import './hero.css';
if(window.initialHeroTheme)document.documentElement.className=window.initialHeroTheme;
const root=createRoot(document.getElementById('root'));
Object.assign(window,{renderHero:enabled=>root.render(<StrictMode><div className="pt-16"><Hero canvasEnabled={enabled}/></div></StrictMode>),unmountHero:()=>root.unmount()});
window.renderHero(!new URLSearchParams(location.search).has('no-canvas'));
