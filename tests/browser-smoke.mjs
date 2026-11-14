import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { existsSync } from 'node:fs';
import assert from 'node:assert/strict';
const directory=await mkdtemp(join(tmpdir(),'pastimer-browser-'));
const server=spawn(process.env.PASTIMER_BIN || resolve('target/debug/pastimer'),[],{
  env:{...process.env,DATABASE_PATH:join(directory,'test.sqlite'),DEV_AUTH:'1',PASTIMER_ENV:'development',NODE_ENV:'development',HOST:'127.0.0.1',PORT:'0'},
  stdio:['ignore','pipe','inherit']
});
let browser;
try {
  const base=await new Promise((resolve,reject)=>{
    const timeout=setTimeout(()=>reject(new Error('Rust server startup timed out')),15000);
    let output='';
    server.stdout.on('data',chunk=>{output+=chunk;const match=output.match(/Pastimer listening on (http:\/\/[^\s]+)/);if(match){clearTimeout(timeout);resolve(match[1]);}});
    server.once('error',error=>{clearTimeout(timeout);reject(error);});
    server.once('exit',code=>{clearTimeout(timeout);reject(new Error(`Rust server exited: ${code}`));});
  });
browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? (existsSync('/usr/bin/chromium') ? '/usr/bin/chromium' : undefined), headless: true });
  const errors=[];
  const contexts=await Promise.all([browser.newContext({viewport:{width:390,height:844}}),browser.newContext()]);
  const pages=await Promise.all(contexts.map(async context=>{
    // Local mode does not need Telegram's SDK; isolate external connectivity from UI validation.
    await context.route('https://telegram.org/**',route=>route.abort());
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));
    return page;
  }));
  const [a,b]=pages;
  for(const [page,name] of [[a,'Alice'],[b,'Bob']]){
    await page.goto(base);
    await page.getByLabel('Your name').fill(name);
    await page.getByRole('button',{name:'Continue locally'}).click();
    await page.getByRole('button',{name:'Create a table'}).waitFor();
  }
  await a.locator('#games').selectOption('high-card-demo');
  await a.getByRole('button',{name:'Create a table'}).click();
  await a.locator('#room-code').waitFor({state:'visible'});
  const code=await a.locator('#room-code').textContent();
  await b.getByLabel('Table code').fill(code);
  await b.getByRole('button',{name:'Join table'}).click();
  await a.locator('#seats .seat').nth(1).waitFor();
  await a.getByRole('button',{name:'Start game'}).click();
  for(const page of pages) await page.getByRole('button',{name:'Save & acknowledge deck'}).click();
  for(const page of pages) await page.locator('#hand .card').first().waitFor();
  assert.equal(await a.locator('#hand .card').count(),5);
  await a.screenshot({path:'/tmp/pastimer-mobile.png',fullPage:true});
  // Hold a background refresh open: local card selection must still respond to a tap.
  let releaseRefresh,markRefreshStarted;
  const refreshHeld=new Promise(resolve=>{releaseRefresh=resolve;});
  const refreshStarted=new Promise(resolve=>{markRefreshStarted=resolve;});
  await a.route(`${base}/api/rooms/${code}`,async route=>{
    const response=await route.fetch();markRefreshStarted();await refreshHeld;await route.fulfill({response});
  },{times:1});
  await refreshStarted;
  try{
    await a.locator('#hand .card').first().click();
    assert.equal(await a.locator('#hand .card').first().getAttribute('aria-pressed'),'true','Card selection must work during a background refresh');
    assert.equal(await a.getByRole('button',{name:'Play selected card'}).isEnabled(),true);
  }finally{releaseRefresh();}
  for(let round=0;round<5;round++){
    for(const page of pages){
      const loseResponse=round===0 && page===a;
      if(loseResponse) await page.route('**/api/rooms/*/commands',async route=>{await route.fetch();await route.abort();},{times:1});
      await page.locator('#hand .card').first().click();
      await page.getByRole('button',{name:'Play selected card'}).click();
      try { await page.waitForFunction(expected=>document.querySelectorAll('#hand .card').length===expected,4-round,{timeout:10000}); } catch(e) {console.log('NOTICE',await page.locator('#notice').textContent());await page.screenshot({path:'/tmp/pastimer-failure.png',fullPage:true});throw e;}
      if(loseResponse){
        await page.getByRole('button',{name:'Retry previous move'}).waitFor();
        await page.reload();
        await page.getByRole('button',{name:'Retry previous move'}).click();
        await page.getByRole('button',{name:'Retry previous move'}).waitFor({state:'hidden'});
        assert.equal(await page.locator('#hand .card').count(),4);
      }
    }
    if(round<4) for(const page of pages) await page.getByRole('button',{name:'Play selected card'}).waitFor();
  }
  await a.getByText('Game complete.',{exact:false}).waitFor();
  const result=await a.evaluate(async code=>{
    const audit=await (await fetch(`/api/rooms/${code}/audit`)).json();
    return {phase:audit.phase,moves:audit.commands.length,proof:audit.decks[0].proof.cards.length};
  },code);
  assert.deepEqual(result,{phase:'finished',moves:10,proof:52});
  await a.reload();await a.getByText('Game complete.',{exact:false}).waitFor();
  assert.deepEqual(errors,[]);
  console.log('PASS: two independent browsers, mobile hand, join/start/ack, all 10 moves, final proof, lost-response retry across reload, no browser errors');
}finally{
  if(browser) await browser.close();
  if(server.exitCode===null){const exited=once(server,'exit');server.kill('SIGTERM');await exited;}
  await rm(directory,{recursive:true,force:true});
}
