import { chromium } from 'playwright';
import { spawn,execFileSync } from 'node:child_process';
import { createHmac, randomBytes } from 'node:crypto';
import { once } from 'node:events';
import { mkdtemp,rm,writeFile,mkdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join,resolve,dirname } from 'node:path';
import { existsSync } from 'node:fs';
import assert from 'node:assert/strict';
const output=resolve('artifacts/screenshots');await mkdir(output,{recursive:true});
const botToken=randomBytes(32).toString('hex');
function initData(id,name){const fields=new URLSearchParams({auth_date:String(Math.floor(Date.now()/1000)),user:JSON.stringify({id,first_name:name})});const check=[...fields.entries()].sort(([a],[b])=>a.localeCompare(b)).map(([k,v])=>`${k}=${v}`).join('\n');const key=createHmac('sha256','WebAppData').update(botToken).digest();fields.set('hash',createHmac('sha256',key).update(check).digest('hex'));return fields.toString();}
const directory=await mkdtemp(join(tmpdir(),'pastimer-thousand-'));
const binary=process.env.PASTIMER_BIN||resolve('target/debug/pastimer');
const server=spawn(binary,[],{env:{...process.env,DATABASE_PATH:join(directory,'test.sqlite'),DEV_AUTH:'0',TELEGRAM_BOT_TOKEN:botToken,PASTIMER_ENV:'development',NODE_ENV:'development',HOST:'127.0.0.1',PORT:'0'},stdio:['ignore','pipe','inherit']});
let browser;
try{
  const base=await new Promise((resolve,reject)=>{let output='';const timer=setTimeout(()=>reject(new Error('Server startup timed out')),15000);server.stdout.on('data',chunk=>{output+=chunk;const match=output.match(/Pastimer listening on (http:\/\/[^\s]+)/);if(match){clearTimeout(timer);resolve(match[1]);}});server.once('error',reject);server.once('exit',code=>{clearTimeout(timer);reject(new Error(`Server exit ${code}`));});});
  browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH??(existsSync('/usr/bin/chromium')?'/usr/bin/chromium':undefined),headless:true});
  for(const count of [3]){
    const contexts=[],pages=[],byId=new Map(),errors=[];
    for(let i=0;i<count;i++){
      const context=await browser.newContext({viewport:{width:430,height:932},deviceScaleFactor:2});contexts.push(context);await context.route('https://telegram.org/**',route=>route.abort());
      const name=['Миша','Аня','Саша'][i];
      await context.addInitScript(data=>{window.Telegram={WebApp:{initData:data,ready(){},expand(){}}};},initData(10001+i,name));
      const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));pages.push(page);await page.goto(base);
      await page.getByRole('button',{name:'Create a table'}).waitFor();
      const me=await page.evaluate(async()=>await(await fetch('/api/me')).json());byId.set(me.id,page);
    }
    const host=pages[0];await host.locator('select[name=playerCount]').selectOption(String(count));await host.getByRole('button',{name:'Create a table'}).click();await host.locator('#room-code').waitFor({state:'visible'});const code=await host.locator('#room-code').textContent();
    for(const page of pages.slice(1)){await page.getByLabel('Table code').fill(code);await page.getByRole('button',{name:'Join table'}).click();await page.locator('#room').waitFor({state:'visible'});}
    const read=async page=>await page.evaluate(async code=>await(await fetch(`/api/rooms/${code}`)).json(),code);
    const act=async(page,locator)=>{await locator.waitFor();const version=Number(await page.locator('#room').getAttribute('data-version'));await locator.click();await page.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>v,version);};
    await host.getByRole('button',{name:'Start game'}).waitFor({state:'visible'});await act(host,host.getByRole('button',{name:'Start game'}));let room=await read(host);const dealer=byId.get(room.table.dealer);
    await act(dealer,dealer.getByRole('button',{name:'Shuffle & deal',exact:true}));room=await read(host);assert.equal(room.phase,'playing');assert.equal(room.table.phase,'auction');assert.equal(room.deck.acknowledgements.length,0);
    await byId.get(room.table.turn).getByRole('button',{name:'Pass',exact:true}).waitFor();
    await byId.get(room.table.turn).screenshot({path:join(output,'01-bidding.png'),fullPage:true});
    let auctionSteps=0;
    while(room.table.phase==='auction'){
      const page=byId.get(room.table.turn);await act(page,count===4&&auctionSteps===1?page.getByRole('button',{name:/Here ·/}):page.getByRole('button',{name:'Pass',exact:true}));auctionSteps++;room=await read(host);
    }
    const owner=byId.get(room.table.contractor);await act(owner,owner.getByRole('button',{name:'Take stock',exact:true}));
    await owner.getByLabel('Give openly').first().check();
    await owner.screenshot({path:join(output,'02-distribution.png'),fullPage:true});
    await act(owner,owner.getByRole('button',{name:'Give cards',exact:true}));
    if(count===4){
      await act(owner,owner.getByRole('button',{name:'Take a stick · палочка',exact:true}));room=await read(host);assert.equal(room.table.lastDeal.status,'dismissed');
    }else{
      await owner.getByRole('spinbutton',{name:'Set order & start',exact:true}).fill('105');await act(owner,owner.getByRole('button',{name:'Set order & start',exact:true}));
      for(let i=0;i<24;i++){
        room=await read(host);const page=byId.get(room.table.turn);assert.ok(page,'Turn belongs to a player');

        await page.getByRole('button',{name:'Play selected card',exact:true}).waitFor();
        await page.locator('#hand .card:not(:disabled)').first().click();
        if(i===2)await page.screenshot({path:join(output,'03-playing-a-trick.png'),fullPage:true});
        await act(page,page.getByRole('button',{name:'Play selected card',exact:true}));

        if(i%3===2){
          const closing=await read(page);assert.equal(closing.table.phase,'closing');assert.ok(closing.table.closeAt-closing.table.serverNow>1500,'The trick remains open before collection');
          await page.waitForFunction(()=>document.getElementById('room').dataset.phase!=='closing',null,{timeout:6500});
        }
      }
      room=await read(host);assert.equal(room.table.lastDeal.status,'completed');
      for(const page of pages){await page.locator('#trick-piles .pile').first().waitFor();for(const pile of await page.locator('#trick-piles .pile').all()) await pile.locator('summary').click();}
      await host.waitForFunction(()=>document.getElementById('room').dataset.phase==='awaitShuffle');
      await host.screenshot({path:join(output,'04-collected-tricks.png'),fullPage:true});
    }
    const audit=await host.evaluate(async code=>await(await fetch(`/api/rooms/${code}/audit`)).json(),code);
    assert.equal(audit.deals.length,1);assert.equal(audit.decks[0].proof.cards.length,24);assert.ok(audit.deals[0].movements.length>=29);
    for(const pile of room.table.piles){const last=audit.deals[0].tricks.filter(t=>t.winner===pile.player).at(-1)??null;assert.deepEqual(pile.top,last);}
    const file=join(directory,`audit-${count}.json`);await writeFile(file,JSON.stringify(audit));const verified=JSON.parse(execFileSync(join(dirname(binary),'verify-audit'),[file],{encoding:'utf8'}));assert.equal(verified.valid,true);assert.equal(verified.deals,1);
    await host.reload();await host.locator('#trick-piles').waitFor({state:'visible'});assert.equal((await read(host)).table.auditDeals,1);assert.deepEqual(errors,[]);
    console.log(`Saved four mobile Mini App previews to ${output}; signed test Telegram identities, real server rules and verified audit.`);
    for(const context of contexts)await context.close();
  }
}finally{
  if(browser)await browser.close();if(server.exitCode===null){const exited=once(server,'exit');server.kill('SIGTERM');await exited;}await rm(directory,{recursive:true,force:true});
}
