import {expect,test} from "bun:test";
import {mkdirSync} from "node:fs";
import {resolve} from "node:path";
import {createElement} from "react";
import {renderToStaticMarkup} from "react-dom/server";
import {InlineCode,stripBackticks} from "../crates/web/src/inline_code.jsx";
import {HeroTabProvider,useHeroTab} from "../crates/web/src/hero_tab_state.jsx";
import {HeroTabs} from "../crates/web/src/hero_tabs.jsx";
import {VgpuWordmarkGlyphs} from "../crates/web/src/wordmark.jsx";

// Package only the reference leaves for test execution. No reference TS is
// imported by the application or included in a production bundle.
const root=resolve(import.meta.dir,"..");
const upstream=resolve(process.env.VGPU_UPSTREAM_DIR??"../vgpu");
const output=resolve(root,"node_modules/.cache/vgpu-reference-ui");
mkdirSync(output,{recursive:true});
const built=await Bun.build({
  entrypoints:[
    resolve(upstream,"apps/docs/app/[lang]/(home)/components/inline-code.tsx"),
    resolve(upstream,"apps/docs/components/vgpu-wordmark.tsx"),
    resolve(upstream,"apps/docs/components/hero/hero-tab-state.tsx"),
    resolve(upstream,"apps/docs/app/[lang]/(home)/components/hero-tabs.tsx"),
  ],
  outdir:output,
  external:["react","react/jsx-runtime","lucide-react"],
  plugins:[{name:"reference-hero-context",setup(build){
    build.onResolve({filter:/^@\/components\/hero\/hero-tab-state$/},()=>({path:resolve(upstream,"apps/docs/components/hero/hero-tab-state.tsx")}));
  }}],
  target:"browser",
  format:"esm",
  splitting:true,
});
if(!built.success)throw new AggregateError(built.logs,"Cannot build upstream reference leaves");
const inlineOutput=built.outputs.find(asset=>asset.path.endsWith("/inline-code.js"));
const wordmarkOutput=built.outputs.find(asset=>asset.path.endsWith("/vgpu-wordmark.js"));
if(!inlineOutput || !wordmarkOutput) throw new Error("Reference leaves missing from build outputs");
const reference=await import(inlineOutput.path);
const wordmark=await import(wordmarkOutput.path);
const stateOutput=built.outputs.find(asset=>asset.path.endsWith("/hero-tab-state.js"));
if(!stateOutput)throw new Error("Reference hero context missing from build");
const tabState=await import(stateOutput.path);

test("Rust inline literals render exactly the original HTML",()=>{
  for(const text of ["","plain prose","a `b` c","`pnpm add vgpu`","`one` and `two`","an `unfinished","a&b <c> `x`"]) {
    expect(stripBackticks(text)).toBe(reference.stripBackticks(text));
    for(const mono of [true,false])for(const wrap of [true,false]){
      const props={text,mono,wrap};
      expect(renderToStaticMarkup(createElement(InlineCode,props)))
        .toBe(renderToStaticMarkup(createElement(reference.InlineCode,props)));
    }
  }
});

test("Rust wordmark preserves every original SVG path and transform",()=>{
  const port=renderToStaticMarkup(createElement("svg",{},createElement(VgpuWordmarkGlyphs)));
  const original=renderToStaticMarkup(createElement("svg",{},createElement(wordmark.VgpuWordmarkGlyphs)));
  expect(port).toBe(original);
});


test("Rust hero context preserves the initial tab, setter and missing-provider error",()=>{
  function PortProbe(){const state=useHeroTab();return createElement("span",{},state.activeTab,typeof state.setActiveTab);}
  function ReferenceProbe(){const state=tabState.useHeroTab();return createElement("span",{},state.activeTab,typeof state.setActiveTab);}
  const actual=renderToStaticMarkup(createElement(HeroTabProvider,{children:createElement(PortProbe)}));
  const expected=renderToStaticMarkup(createElement(tabState.HeroTabProvider,{children:createElement(ReferenceProbe)}));
  expect(actual).toBe(expected);
  expect(actual).toBe("<span>Promptfunction</span>");
  for(const Probe of [PortProbe,ReferenceProbe]){
    let thrown:unknown;
    try{renderToStaticMarkup(createElement(Probe));}catch(error){thrown=error;}
    expect(thrown).toBeInstanceOf(Error);
    expect((thrown as Error).message).toBe("useHeroTab must be used inside HeroTabProvider.");
  }
});

const tabsOutput=built.outputs.find(asset=>asset.path.endsWith("/hero-tabs.js"));
if(!tabsOutput)throw new Error("Reference hero tabs missing from build");
const referenceTabs=await import(tabsOutput.path);
test("Rust setup tabs preserve the exact initial upstream markup and icon",()=>{
  const port=renderToStaticMarkup(createElement(HeroTabProvider,{children:createElement(HeroTabs)}));
  const original=renderToStaticMarkup(createElement(tabState.HeroTabProvider,{children:createElement(referenceTabs.HeroTabs)}));
  expect(port).toBe(original);
});
