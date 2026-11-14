// Test mounting only. Application behavior remains in compiled Rust.
import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import {HeroTabProvider} from '../../../crates/web/src/hero_tab_state.jsx';
import {HeroTabs} from '../../../crates/web/src/hero_tabs.jsx';
import './style.css';
const root=createRoot(document.getElementById('root'));
// Test-only lifecycle seam on the mounting boundary, absent from app modules.
Object.assign(window,{unmountFixture:()=>root.unmount()});
root.render(
  <StrictMode><section data-hero-theme><HeroTabProvider><HeroTabs /></HeroTabProvider></section></StrictMode>,
);
