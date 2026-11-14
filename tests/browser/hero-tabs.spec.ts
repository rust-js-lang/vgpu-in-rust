import {expect,test,type Page} from '@playwright/test';
const snippets={
  Prompt:'Setup vgpu on my project, run npx vgpu',
  CLI:'pnpm add vgpu',
  Skill:'npx skills add vercel-labs/vgpu',
  MCP:'npx -y add-mcp https://vgpu.sh/api/mcp -g',
};
const active=(page:Page)=>page.locator('[data-hero-tabs-copy] .grid > [aria-hidden="false"]');
for(const [implementation,url] of [['Rust','/'],['Upstream','/reference.html']] as const){
  test.describe(implementation,()=>{
    test('all snippets, copy confirmations, stable layout and rapid transitions',async({page})=>{
      const errors:string[]=[];
      page.on('pageerror',error=>errors.push(error.message));
      await page.addInitScript(()=>{
        Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText(text:string){Reflect.set(window,'__copiedText',text);return Promise.resolve();}}});
      });
      await page.goto(url);
      await expect(active(page)).toHaveText(snippets.Prompt);
      await page.clock.install();
      const copy=page.locator('[data-hero-tabs-copy]');
      const initial=(await copy.boundingBox())!;
      for(const [tab,text] of Object.entries(snippets)){
        await page.getByRole('tab',{name:tab,exact:true}).click();
        await expect(page.getByRole('tab',{name:tab,exact:true})).toHaveAttribute('aria-selected','true');
        await expect(active(page)).toHaveText(text);
        await expect(copy).toHaveAccessibleName('Copy: '+text);
        const current=(await copy.boundingBox())!;
        expect(current.height).toBe(initial.height);
        expect(current.y).toBe(initial.y);
        await copy.click();
        await expect(copy).toHaveAccessibleName('Copied');
        expect(await page.evaluate(()=>Reflect.get(window,'__copiedText'))).toBe(text);
        await page.clock.runFor(1601);
        await expect(copy).toHaveAccessibleName('Copy: '+text);
      }
      await page.getByRole('tab',{name:'Prompt',exact:true}).click();
      await page.clock.runFor(151);
      await page.getByRole('tab',{name:'CLI',exact:true}).click();
      await expect(page.locator('.grid > span').filter({hasText:snippets.Prompt})).toHaveClass(/-translate-y-1\.5/);
      await page.clock.runFor(100);
      await page.getByRole('tab',{name:'Skill',exact:true}).click();
      await page.clock.runFor(60);
      const leaving=page.locator('.grid > span').filter({hasText:snippets.CLI});
      await expect(leaving).toHaveClass(/-translate-y-1\.5/);
      await page.clock.runFor(91);
      await expect(leaving).toHaveClass(/^.*\stranslate-y-1\.5\s/);
      await expect(leaving).not.toHaveClass(/duration-150/);
      expect(errors).toEqual([]);
    });

    for(const failure of ['denied','missing'] as const){
      test('clipboard '+failure+' fails silently and leaves selectable text',async({page})=>{
        const errors:string[]=[];
        page.on('pageerror',error=>errors.push(error.message));
        await page.addInitScript(failure=>{
          Object.defineProperty(navigator,'clipboard',{configurable:true,value:failure==='missing'?undefined:{writeText(){return Promise.reject(new DOMException('Denied','NotAllowedError'));}}});
        },failure);
        await page.goto(url);
        const copy=page.locator('[data-hero-tabs-copy]');
        await copy.click();
        await expect(copy).toHaveAccessibleName('Copy: '+snippets.Prompt);
        await page.getByRole('tab',{name:'CLI',exact:true}).click();
        await expect(active(page)).toHaveText(snippets.CLI);
        await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve()))));
        expect(errors).toEqual([]);
      });
    }

    test('unmount clears the outstanding outgoing-layer timer',async({page})=>{
      await page.addInitScript(()=>{
        const pending=new Set<number>();
        const schedule=window.setTimeout.bind(window),clear=window.clearTimeout.bind(window);
        Object.assign(window,{pendingParkTimers:pending});
        window.setTimeout=((handler:TimerHandler,ms?:number,...args:unknown[])=>{
          const id=schedule(handler,ms,...args);
          if(ms===150)pending.add(id);
          return id;
        }) as typeof setTimeout;
        window.clearTimeout=(id?:number)=>{pending.delete(id!);clear(id);};
      });
      await page.goto(url);
      await expect(active(page)).toHaveText(snippets.Prompt);
      // Inspect scheduling and unmount in one task, before the real 150ms timer
      // can run. Fake clocks replace the timer wrapper being observed here.
      const counts=await page.evaluate(()=>{
        const tab=[...document.querySelectorAll<HTMLButtonElement>('[role="tab"]')].find(tab=>tab.textContent==='CLI')!;
        tab.click();
        const pending=Reflect.get(window,'pendingParkTimers') as Set<number>;
        const before=pending.size;
        Reflect.get(window,'unmountFixture')();
        return [before,pending.size];
      });
      expect(counts).toEqual([1,0]);
      await expect(page.locator('[data-hero-tabs]')).toHaveCount(0);
    });

    test('reduced motion preserves final states without transitions on mobile',async({page})=>{
      await page.setViewportSize({width:375,height:812});
      await page.emulateMedia({reducedMotion:'reduce'});
      await page.goto(url);
      await page.getByRole('tab',{name:'MCP',exact:true}).click();
      await expect(active(page)).toHaveText(snippets.MCP);
      expect(await active(page).evaluate(element=>getComputedStyle(element).transitionProperty)).toBe('none');
      const bounds=(await page.locator('[data-hero-tabs]').boundingBox())!;
      expect(bounds.x).toBeGreaterThanOrEqual(0);
      expect(bounds.x+bounds.width).toBeLessThanOrEqual(375);
    });
  });
}
