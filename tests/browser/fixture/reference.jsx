// Upstream TS is imported only in this test fixture, never the application.
import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import {HeroTabProvider} from '@reference/components/hero/hero-tab-state';
import {HeroTabs} from '@reference/app/[lang]/(home)/components/hero-tabs';
import './style.css';
const root=createRoot(document.getElementById('root'));
// Test-only lifecycle seam on the mounting boundary, absent from app modules.
Object.assign(window,{unmountFixture:()=>root.unmount()});
root.render(
  <StrictMode><section data-hero-theme><HeroTabProvider><HeroTabs /></HeroTabProvider></section></StrictMode>,
);
