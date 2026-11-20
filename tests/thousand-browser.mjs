import {chromium} from 'playwright';
import {spawn,execFileSync} from 'node:child_process';
import {once} from 'node:events';
import {mkdtemp,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve,dirname} from 'node:path';
import {existsSync} from 'node:fs';
import assert from 'node:assert/strict';
const directory=await mkdtemp(join(tmpdir(),'pastimer-thousand-')),binary=process.env.PASTIMER_BIN||resolve('target/debug/pastimer');
const server=spawn(binary,[],{env:{...process.env,DATABASE_PATH:join(directory,'test.sqlite'),DEV_AUTH:'1',PASTIMER_ENV:'development',NODE_ENV:'development',HOST:'127.0.0.1',PORT:'0'},stdio:['ignore','pipe','inherit']});
let browser;
async function dragCard(page,card,target,touch=false,cancel=false){
  const source=await card.boundingBox(),destination=await target.boundingBox();assert.ok(source&&destination);
  const start={x:source.x+8,y:source.y+18},end={x:destination.x+destination.width/2,y:destination.y+destination.height/2};
  if(touch){const session=await page.context().newCDPSession(page);await session.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{...start,id:1}]});for(let i=1;i<=12;i++)await session.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:start.x+(end.x-start.x)*i/12,y:start.y+(end.y-start.y)*i/12,id:1}]});await session.send('Input.dispatchTouchEvent',{type:cancel?'touchCancel':'touchEnd',touchPoints:[]});await session.detach();}
  else{await page.mouse.move(start.x,start.y);await page.mouse.down();await page.mouse.move(end.x,end.y,{steps:12});await page.mouse.up();}
  assert.equal(await page.locator('.transfer-drag').count(),0,'Drag ghost is removed after dropping');
}

async function fits(page){
  const bad=await page.evaluate(()=>{
    const nodes=[document.getElementById('room'),...document.querySelectorAll('#hand .card,#game-actions button,#table-actions button,#thousand-table .playing-card,.trump-marker,.incoming-card,.seat-transfer')];
    return nodes.filter(e=>{const r=e.getBoundingClientRect();return r.width&&r.height&&(r.left<-.5||r.right>innerWidth+.5||r.top<-.5||r.bottom>innerHeight+.5);}).map(e=>({tag:e.tagName,cls:e.className,rect:JSON.stringify(e.getBoundingClientRect())}));
  });if(bad.length){await page.screenshot({path:'/tmp/pastimer-fit-failure.png',fullPage:true});console.log('FIT',await page.evaluate(()=>({width:innerWidth,height:innerHeight,hand:document.getElementById('hand').getBoundingClientRect().toJSON(),overlap:document.getElementById('hand').style.getPropertyValue('--overlap'),card:document.getElementById('hand').style.getPropertyValue('--card-width'),room:document.getElementById('room').getBoundingClientRect().toJSON()})));}assert.deepEqual(bad,[],'Normal gameplay controls and cards must fit the visible screen');
}
try{
  const base=await new Promise((resolve,reject)=>{let output='';const timer=setTimeout(()=>reject(new Error('Server startup timed out')),15000);server.stdout.on('data',chunk=>{output+=chunk;const m=output.match(/Pastimer listening on (http:\/\/[^\s]+)/);if(m){clearTimeout(timer);resolve(m[1]);}});server.once('error',reject);});
  browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH??(existsSync('/usr/bin/chromium')?'/usr/bin/chromium':undefined),headless:true});
  for(const count of [3,4]){
    const contexts=[],pages=[],byId=new Map(),errors=[];
    for(let i=0;i<count;i++){
      const context=await browser.newContext({viewport:{width:390,height:650},hasTouch:true});contexts.push(context);await context.route('https://telegram.org/**',r=>r.abort());
      const page=await context.newPage();pages.push(page);page.on('pageerror',e=>{errors.push(e.message);console.error('PAGE ERROR',e.message);});await page.goto(base);await page.getByLabel('Your name').fill(`Player ${i+1} with a longer name`);await page.getByRole('button',{name:'Continue locally'}).click();await page.getByRole('button',{name:'Create a table'}).waitFor();
      const me=await page.evaluate(async()=>await(await fetch('/api/me')).json());byId.set(me.id,page);
    }
    const host=pages[0];await contexts[0].grantPermissions(['clipboard-read','clipboard-write'],{origin:base});await host.getByRole('button',{name:`${count} seats`,exact:true}).click();await host.getByRole('button',{name:'Create a table'}).click();await host.locator('#room-code').waitFor({state:'visible'});const code=await host.locator('#room-code').textContent();
    for(const page of pages.slice(1)){await page.getByLabel('Table code').fill(code);await page.getByRole('button',{name:'Join table'}).click();await page.locator('#room').waitFor({state:'visible'});}
    await host.getByRole('button',{name:'Copy invitation code',exact:true}).click();assert.equal(await host.evaluate(()=>navigator.clipboard.readText()),code);
    const read=async page=>await page.evaluate(async code=>await(await fetch(`/api/rooms/${code}`)).json(),code);
    const act=async(page,locator)=>{await locator.waitFor();const options=await locator.evaluate(e=>e.classList.contains('card'))?{position:{x:8,y:18}}:{};await locator.click({...options,trial:true}).catch(async error=>{await page.screenshot({path:'/tmp/pastimer-action-failure.png',timeout:5000}).catch(()=>{});console.log('ACTION FAILED',await page.locator('#phase').textContent(),await page.locator('#notice').textContent());throw error;});const version=Number(await page.locator('#room').getAttribute('data-version'));await locator.click(options);await page.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>v,version);};
    await act(host,host.getByRole('button',{name:'Start game',exact:true}));let room=await read(host);const dealer=byId.get(room.table.dealer);await act(dealer,dealer.getByRole('button',{name:'Shuffle & deal',exact:true}));room=await read(host);assert.equal(room.deck.acknowledgements.length,0);
    for(const page of pages){const initial=await read(page);if(initial.table.hand.length){assert.equal(initial.table.hand.length,7,'Deal seven cards to every active seat');await page.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>=v,initial.version);assert.equal(await page.locator('#hand-count').textContent(),'7 cards');assert.equal(await page.locator('#hand .card').count(),7);}}
    const layouts=async page=>{const previous=page.viewportSize();for(const viewport of [{width:320,height:480},{width:320,height:568},{width:390,height:650},{width:430,height:932},{width:568,height:320},{width:844,height:390},{width:1024,height:768}]){await page.setViewportSize(viewport);await page.evaluate(()=>new Promise(r=>requestAnimationFrame(r)));await fits(page);}await page.setViewportSize(previous);};
    await layouts(host);
    await host.setViewportSize({width:390,height:650});let steps=0;
    while(room.table.phase==='auction'){
      const page=byId.get(room.table.turn);await act(page,count===4&&steps===1?page.getByRole('button',{name:/Here -/}):page.getByRole('button',{name:'Pass',exact:true}));steps++;room=await read(host);
      if(room.table.phase==='auction'){await page.locator('.call-bubble').first().waitFor();assert.ok(await page.locator('.call-bubble').count()>=1);}
    }
    const owner=byId.get(room.table.contractor);await owner.getByRole('button',{name:'Take stock',exact:true}).waitFor().catch(async error=>{await owner.screenshot({path:'/tmp/pastimer-stock-failure.png'});console.log('STOCK',await owner.locator('#phase').textContent(),await owner.locator('#notice').textContent());throw error;});assert.equal(await owner.locator('.call-bubble').count(),0,'Auction calls clear once the last call finishes and the auction ends');
    assert.equal(await owner.locator('.stock-row .playing-card').count(),3);assert.ok((await owner.locator('.stock-row .playing-card').first().boundingBox()).width>=65);await layouts(owner);await owner.screenshot({path:'/tmp/pastimer-open-stock.png'});
    await act(owner,owner.getByRole('button',{name:'Take stock',exact:true}));assert.equal(await owner.locator('.stock-display').count(),0,'Taken stock disappears');
    const give=owner.getByRole('button',{name:'Give two cards',exact:true});assert.equal(await give.isDisabled(),true);
    const targets=owner.locator('[data-recipient] .seat-hand');assert.equal(await targets.count(),2);
    assert.equal(await owner.locator('.distribution-form,.transfer-grid').count(),0,'No separate distribution panel');
    const firstCard=await owner.locator('#hand .card').first().getAttribute('data-card');
    await dragCard(owner,owner.locator('#hand .card').first(),owner.locator('#phase'));assert.equal(await owner.locator('#hand .card').count(),10,'Invalid drops keep the card');
    await dragCard(owner,owner.locator('#hand .card').first(),targets.nth(0),true,true);assert.equal(await owner.locator('#hand .card').count(),10,'Cancelled touch gestures keep the card');
    await dragCard(owner,owner.locator('#hand .card').first(),targets.nth(0));assert.equal(await owner.locator('.seat-transfer').count(),1);assert.equal(await owner.locator('#hand .card').count(),9);assert.equal(await give.isDisabled(),true);
    await dragCard(owner,owner.locator('.seat-transfer').first(),owner.locator('#hand'));assert.equal(await owner.locator('.seat-transfer').count(),0);assert.equal(await owner.locator('#hand .card').count(),10,'Drag back to hand cancels the transfer');
    await dragCard(owner,owner.locator(`#hand .card[data-card="${firstCard}"]`),targets.nth(0));
    await dragCard(owner,owner.locator('#hand .card').first(),targets.nth(1),true);assert.equal(await owner.locator('.seat-transfer').count(),2,'Touch and mouse both stage cards');assert.equal(await owner.locator('#hand .card').count(),8);assert.equal(await give.isEnabled(),true);
    const face=owner.locator('.seat-transfer').first();assert.equal(await face.getAttribute('data-visibility'),'secret');await face.click();assert.equal(await face.getAttribute('data-visibility'),'open');await face.click();assert.equal(await face.getAttribute('data-visibility'),'secret');await face.click();assert.equal(await face.getAttribute('data-visibility'),'open');
    await layouts(owner);await owner.screenshot({path:'/tmp/pastimer-distribution.png'});await act(owner,give);room=await read(owner);
    let secretReceipts=0,publicReceipts=0;
    for(const page of pages){
      const own=await read(page),myId=[...byId].find(([,p])=>p===page)?.[0];
      await page.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>=v,own.version);
      for(const delivery of own.table.deliveries)if(!delivery.public&&delivery.from!==myId&&delivery.to!==myId)assert.equal(delivery.card,null,'Private transfers stay hidden from the third player');
      while(await page.locator('.incoming-card').count()){
        assert.equal(own.table.hand.length,8,'A stock recipient holds eight cards before playing');assert.equal(await page.locator('#hand-count').textContent(),'8 cards');
        const gift=page.locator('.incoming-card').first(),label=await gift.getAttribute('aria-label'),before=await page.locator('#hand .card').count();
        assert.doesNotMatch(await gift.textContent(),/from |tap to/i,'Receipt is highlighted without instructional text');assert.equal(await page.locator('#sort-hand').isVisible(),false);assert.equal(await page.locator('#game-actions button:not(:disabled),#hand .card:not(:disabled)').count(),0,'The incoming card is the only enabled gameplay action');await layouts(page);
        if(label.startsWith('Reveal')){await page.screenshot({path:'/tmp/pastimer-secret-card.png'});secretReceipts++;assert.equal(await gift.locator('[data-card]').count(),0);await gift.click();assert.equal(await page.locator('#hand .card').count(),before);await page.locator('.incoming-card').first().click();}
        else{publicReceipts++;assert.equal(await gift.locator('[data-card]').count(),1);await gift.click();}
        assert.equal(await page.locator('#hand .card').count(),before+1);
      }
    }
    assert.equal(secretReceipts,1);assert.equal(publicReceipts,1);
    if(count===4)assert.equal(await owner.getByRole('button',{name:'Take a stick',exact:true}).isEnabled(),true);
    {
      await owner.getByRole('spinbutton',{name:'Set order & start',exact:true}).fill('105');await act(owner,owner.getByRole('button',{name:'Set order & start',exact:true}));
      for(let i=0;i<24;i++){
        room=await read(host);const page=byId.get(room.table.turn);assert.ok(page);if(process.env.PASTIMER_TRACE)console.log('PLAY',count,i,room.table.phase,room.version);
        if(i===0)await page.route('**/api/rooms/*/commands',async route=>{await route.fetch();await route.abort();},{times:1});
        const own=await read(page),marriage=own.table.controls.announceCards.find(c=>c.endsWith('H'))??own.table.controls.announceCards[0];
        if(marriage)assert.equal(await page.locator('.marriage-toggle[aria-pressed=true]').count()>0,true);
        await act(page,marriage?page.locator(`#hand .card[data-card="${marriage}"]`):page.locator('#hand .card:not(:disabled)').first());assert.equal(await page.getByRole('button',{name:'Play selected card',exact:true}).count(),0);
        if(marriage){const after=await read(page);assert.equal(after.table.trump,marriage.slice(-1));assert.equal(after.table.announcingCard,marriage);assert.equal(await page.locator('.trump-marker').getAttribute('data-trump'),marriage.slice(-1));assert.equal(await page.locator('.card-marriage circle').count(),2);}

        if(i===0){await page.getByRole('button',{name:'Retry previous move'}).waitFor();await page.reload();await page.getByRole('button',{name:'Retry previous move'}).click();await page.getByRole('button',{name:'Retry previous move'}).waitFor({state:'hidden'});}
        if(i===0&&count===4){const inactive=byId.get((await read(page)).table.dealer);await inactive.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>=v,Number(await page.locator('#room').getAttribute('data-version')));await layouts(inactive);assert.equal(await inactive.locator('.trick-play').count(),1,'The sitting-out dealer sees played cards in the free table area');}
        if(i===2){await layouts(page);await page.screenshot({path:'/tmp/pastimer-trick.png'});} else await fits(page);assert.equal(await page.locator('.stock-display').count(),0);
        const cues=await read(page);assert.equal(await page.locator('.hud-pill').count(),0);assert.equal(await page.locator('.seat.is-leader').getAttribute('data-player'),cues.table.trickLeader);
        assert.equal(await page.locator('.trick-play').evaluateAll(slots=>slots.every(slot=>{const seat=[...document.querySelectorAll('.seat')].find(e=>e.dataset.player===slot.dataset.player);return slot.dataset.position===(seat.classList.contains('seat-self')?'self':seat.classList.contains('seat-left')?'left':'right');})),true,'Cards stay with their owners rather than play order');
        if(i%3===2){const closing=await read(page);assert.equal(closing.table.phase,'closing');assert.equal(await page.locator('.seat.is-taking').getAttribute('data-player'),closing.table.trickWinner);assert.equal(await page.locator(`.trick-play[data-collecting-to="${closing.table.trickWinner}"]`).count(),3);const countBefore=closing.table.piles.find(p=>p.player===closing.table.trickWinner).count;await page.waitForFunction(v=>Number(document.getElementById('room').dataset.version)>v,closing.version,{timeout:6500});assert.equal((await read(page)).table.piles.find(p=>p.player===closing.table.trickWinner).count,countBefore+1);}

        if(i===8){
          const recipient=pages.find(p=>p!==owner&&room.table.seats.find(s=>byId.get(s.id)===p)?.count!==0),state=await recipient.context().storageState();
          const fresh=await browser.newContext({viewport:{width:390,height:650},hasTouch:true,storageState:{cookies:state.cookies,origins:[]}});await fresh.route('https://telegram.org/**',r=>r.abort());const reopened=await fresh.newPage();await reopened.goto(base);await reopened.locator('#room-code').waitFor();
          const resumed=await read(reopened);assert.equal(resumed.table.hand.length,5);assert.equal(await reopened.locator('#hand-count').textContent(),'5 cards');assert.equal(await reopened.locator('#hand .card').count(),5);assert.equal(await reopened.locator('.incoming-card').count(),0,'Reopening mid-deal must not replay a stock receipt');await fits(reopened);await fresh.close();
        }
      }
      room=await read(host);assert.equal(room.table.lastDeal.status,'completed');if(count===4){for(const page of pages)assert.equal(await page.locator('.seat-dealer').getAttribute('data-player'),room.table.lastDeal.dealer,'Keep the completed deal participants inspectable after the next dealer rotates');}await host.screenshot({path:'/tmp/pastimer-thousand-mobile.png',fullPage:true});
      for(const page of pages)for(const button of await page.locator('.result-player:not(:disabled)').all()){await button.click();assert.equal(await page.locator('#inspector .playing-card').count(),3);await page.getByRole('button',{name:'Close trick',exact:true}).click();}
    }
    const audit=await host.evaluate(async code=>await(await fetch(`/api/rooms/${code}/audit`)).json(),code);const file=join(directory,`audit-${count}.json`);await writeFile(file,JSON.stringify(audit));const verified=JSON.parse(execFileSync(join(dirname(binary),'verify-audit'),[file],{encoding:'utf8'}));assert.equal(verified.valid,true);assert.equal(verified.deals,1);
    // A fresh Mini App browser window has neither sessionStorage nor localStorage.
    const state=await contexts[0].storageState(),fresh=await browser.newContext({viewport:{width:390,height:650},hasTouch:true,storageState:{cookies:state.cookies,origins:[]}});await fresh.route('https://telegram.org/**',r=>r.abort());const reopened=await fresh.newPage();await reopened.goto(base);await reopened.locator('#room-code').waitFor({state:'visible'});assert.equal(await reopened.locator('#room-code').textContent(),code,'Server membership reconnects a fresh window');assert.equal((await read(reopened)).table.auditDeals,1);await fits(reopened);await fresh.close();assert.deepEqual(errors,[]);
    console.log(`PASS: ${count}-player Thousand, screen bounds, large stock/hide-on-take, hand-selected open/secret transfers, direct play, seat trick inspection, audit and fresh-window reconnect`);
    for(const context of contexts)await context.close();
  }
}finally{if(browser)await browser.close();if(server.exitCode===null){const exited=once(server,'exit');server.kill('SIGTERM');await exited;}await rm(directory,{recursive:true,force:true});}
