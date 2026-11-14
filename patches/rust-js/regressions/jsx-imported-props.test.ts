import {beforeAll,expect,test} from 'bun:test';
import {writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {createElement} from 'react';
import {renderToStaticMarkup} from 'react-dom/server';
import {buildReact,compiler,fixture,run,target} from './support';
beforeAll(buildReact,60_000);
test('ordinary wrappers use imported and renamed flattened props companions',async()=>{
 const dir=fixture('jsx-imported-props');
 writeFileSync(join(dir,'lib.rs'),`use react::{JSX,jsx};
 use react::attributes::AnchorHTMLAttributes;
 use react::attributes::AnchorHTMLAttributes as RenamedProps;
 pub fn Wrapped(props: AnchorHTMLAttributes<'_>) -> JSX::Element { jsx! {<a {...props} />} }
 pub fn Renamed(props: RenamedProps<'_>) -> JSX::Element { jsx! {<a {...props} />} }
 pub fn Qualified(props: react::attributes::AnchorHTMLAttributes<'_>) -> JSX::Element { jsx! {<a {...props} />} }
 pub fn App()->JSX::Element { jsx! {<><Wrapped href={Some("/one")} className={Some("first")} /><Renamed href={Some("/two")} className={Some("second")} /><Qualified href={Some("/three")} className={Some("third")} /></>} }
 `);
 run([compiler,join(dir,'lib.rs'),'-o',join(dir,'lib.js'),'--','--extern',`react=${join(target,'libreact.rmeta')}`,'-L',target]);
 const lib=await import(join(dir,'lib.js'));
 expect(renderToStaticMarkup(createElement(lib.App))).toBe('<a href="/one" class="first"></a><a href="/two" class="second"></a><a href="/three" class="third"></a>');
});
