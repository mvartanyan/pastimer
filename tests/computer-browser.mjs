import {chromium} from 'playwright';
import {spawn,execFileSync} from 'node:child_process';
import {once} from 'node:events';
import {mkdtemp,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve,dirname} from 'node:path';
import {existsSync} from 'node:fs';
import assert from 'node:assert/strict';

const directory=await mkdtemp(join(tmpdir(),'pastimer-computers-'));
const binary=process.env.PASTIMER_BIN||resolve('target/debug/pastimer');
const server=spawn(binary,[],{env:{...process.env,DATABASE_PATH:join(directory,'test.sqlite'),DEV_AUTH:'1',PASTIMER_ENV:'development',NODE_ENV:'development',HOST:'127.0.0.1',PORT:'0'},stdio:['ignore','pipe','inherit']});
let browser;
try{
  const base=await new Promise((resolve,reject)=>{let output='';const timer=setTimeout(()=>reject(new Error('Server startup timed out')),15000);server.stdout.on('data',chunk=>{output+=chunk;const match=output.match(/Pastimer listening on (http:\/\/[^\s]+)/);if(match){clearTimeout(timer);resolve(match[1]);}});server.once('error',reject);server.once('exit',code=>{clearTimeout(timer);reject(new Error(`Server exit ${code}`));});});
  browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH??(existsSync('/usr/bin/chromium')?'/usr/bin/chromium':undefined),headless:true});
  for(const humans of [1,2]){
    const contexts=[],pages=[],byId=new Map(),errors=[];
    for(let i=0;i<humans;i++){
      const context=await browser.newContext({viewport:{width:390,height:844}});contexts.push(context);await context.route('https://telegram.org/**',route=>route.abort());
      const page=await context.newPage();pages.push(page);page.on('pageerror',e=>errors.push(e.message));await page.goto(base);
      await page.getByLabel('Your name').fill(`Human ${i+1}`);await page.getByRole('button',{name:'Continue locally'}).click();await page.getByRole('button',{name:'Create a table'}).waitFor();
      const me=await page.evaluate(async()=>await(await fetch('/api/me')).json());byId.set(me.id,page);
    }
    const host=pages[0];
    await host.getByLabel('Computer seats').selectOption(String(3-humans));
    if(humans===1)assert.match(await host.locator('#computer-note').textContent(),/Just you/);
    else assert.match(await host.locator('#computer-note').textContent(),/Invite 1 friend/);
    await host.getByRole('button',{name:'Create a table'}).click();await host.locator('#room-code').waitFor({state:'visible'});const code=await host.locator('#room-code').textContent();
    if(humans===2){
      assert.equal(await host.getByRole('button',{name:'Start game',exact:true}).isDisabled(),true);
      await pages[1].getByLabel('Table code').fill(code);await pages[1].getByRole('button',{name:'Join table'}).click();
    }
    const read=async page=>await page.evaluate(async code=>await(await fetch(`/api/rooms/${code}`)).json(),code);
    const act=async(page,locator)=>{await locator.waitFor();const version=Number(await page.locator('#room').getAttribute('data-version'));await locator.click(await locator.evaluate(e=>e.classList.contains('card'))?{position:{x:8,y:18}}:{});await page.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>v,version);};
    await act(host,host.getByRole('button',{name:'Start game',exact:true}));
    let room=await read(host);assert.equal(room.computers.length,3-humans);assert.equal(room.players.length,3);
    const dealer=byId.get(room.table.dealer);
    if(dealer)await act(dealer,dealer.getByRole('button',{name:'Shuffle & deal',exact:true}));
    else await act(host,host.getByRole('button',{name:'Ask computer to deal',exact:true}));
    for(let step=0;step<90;step++){
      room=await read(host);
      if(process.env.PASTIMER_TRACE)console.log(JSON.stringify({humans,step,version:room.version,phase:room.table.phase,hand:room.table.hand.length,remaining:room.table.closeAt-room.table.serverNow}));
      if(room.table.auditDeals===1)break;
      for(const p of pages){await p.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>=v,room.version);while(await p.locator('.incoming-card').count())await p.locator('.incoming-card').first().click();}
      if(room.computerDueAt){await host.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>v,room.version,{timeout:Math.max(6500,room.computerDueAt-room.table.serverNow+4500)});continue;}
      assert.equal(room.state,undefined);assert.equal(room.table.hands,undefined);
      if(room.table.phase==='closing'){
        await host.waitForFunction(version=>Number(document.getElementById('room').dataset.version)>version,room.version,{timeout:6500});continue;
      }
      const actor=room.table.turn??room.table.contractor,page=byId.get(actor);
      assert.ok(page,`Computer must progress to a human decision or trick closure, got ${room.table.phase}`);
      if(room.table.phase==='auction')await act(page,page.getByRole('button',{name:'Pass',exact:true}));
      else if(room.table.phase==='stock')await act(page,page.getByRole('button',{name:'Take stock',exact:true}));
      else if(room.table.phase==='distribution'){await page.locator('#hand .card').nth(0).click({position:{x:8,y:18}});await page.locator('#hand .card').nth(1).click({position:{x:8,y:18}});await act(page,page.getByRole('button',{name:'Give two cards',exact:true}));}
      else if(room.table.phase==='preparation')await act(page,page.getByRole('button',{name:'Set order & start',exact:true}));
      else if(room.table.phase==='trick'){
        await act(page,page.locator('#hand .card:not(:disabled)').first());
      }else assert.fail(`Unexpected phase ${room.table.phase}`);
    }
    room=await read(host);for(let i=1;i<(room.auctionCalls??[]).length;i++){const call=room.auctionCalls[i];if(room.computers.includes(call.player))assert.ok(call.at-room.auctionCalls[i-1].at>=2800,'Computer calls must be paced rather than committed in a burst');}assert.equal(room.table.auditDeals,1);assert.equal(room.table.lastDeal.status,'completed');assert.equal(room.table.phase,'awaitShuffle');
    const audit=await host.evaluate(async code=>await(await fetch(`/api/rooms/${code}/audit`)).json(),code);
    const file=join(directory,`audit-${humans}.json`);await writeFile(file,JSON.stringify(audit));const verified=JSON.parse(execFileSync(join(dirname(binary),'verify-audit'),[file],{encoding:'utf8'}));assert.equal(verified.valid,true);assert.equal(verified.deals,1);
    assert.ok(audit.commands.some(c=>room.computers.includes(c.actor)&&c.action.action.type==='play'));
    await host.reload();await host.locator('#room-code').waitFor({state:'visible'});assert.equal((await read(host)).table.auditDeals,1);assert.deepEqual(errors,[]);
    console.log(`PASS: ${humans} human(s), ${3-humans} computer(s), real lobby controls, complete deal, legal automatic turns, verified audit and reload`);
    for(const context of contexts)await context.close();
  }
}finally{
  if(browser)await browser.close();if(server.exitCode===null){const exited=once(server,'exit');server.kill('SIGTERM');await exited;}await rm(directory,{recursive:true,force:true});
}
