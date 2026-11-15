import {chromium} from 'playwright';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {mkdtemp,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {existsSync} from 'node:fs';
import assert from 'node:assert/strict';
const directory=await mkdtemp(join(tmpdir(),'pastimer-durak-browser-'));
const server=spawn(process.env.PASTIMER_BIN||resolve('target/debug/pastimer'),[],{env:{...process.env,DATABASE_PATH:join(directory,'test.sqlite'),DEV_AUTH:'1',PASTIMER_ENV:'development',NODE_ENV:'development',HOST:'127.0.0.1',PORT:'0'},stdio:['ignore','pipe','inherit']});
let browser;
try{
 const base=await new Promise((resolve,reject)=>{let output='';const timer=setTimeout(()=>reject(Error('server startup timeout')),15000);server.stdout.on('data',chunk=>{output+=chunk;const match=output.match(/Pastimer listening on (http:\/\/\S+)/);if(match){clearTimeout(timer);resolve(match[1]);}});server.once('error',reject);server.once('exit',()=>reject(Error('server exited')));});
 browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH??(existsSync('/usr/bin/chromium')?'/usr/bin/chromium':undefined),headless:true});
 const contexts=[],pages=[],errors=[];
 for(let i=0;i<3;i++){const context=await browser.newContext({viewport:{width:390,height:844},hasTouch:true});await context.route('https://telegram.org/**',route=>route.abort());const page=await context.newPage();page.on('pageerror',e=>{errors.push(e.message);console.error('PAGE ERROR',e.message);});await page.goto(base);await page.getByLabel('Your name').fill(`Durak ${i}`);await page.getByRole('button',{name:'Continue locally'}).click();await page.locator('#games option[value="durak-perevodnoy"]').waitFor({state:'attached'});contexts.push(context);pages.push(page);}
 const api=(page,path,body)=>page.evaluate(async({path,body})=>{const response=await fetch(`/api${path}`,body===undefined?{}:{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});const value=await response.json();if(!response.ok)throw Error(JSON.stringify(value));return value;},{path,body});
 const ids=await Promise.all(pages.map(page=>api(page,'/me').then(p=>p.id)));
 for(const game of ['durak-podkidnoy','durak-perevodnoy']){
   const a=pages[0];if(await a.locator('#room').isVisible())await a.locator('#back').click();
   await a.locator('#games').selectOption(game);await a.locator('#durak-seats').selectOption('3');await a.getByRole('button',{name:'Create a table',exact:true}).click();await a.locator('#room-code').waitFor();const code=await a.locator('#room-code').textContent();
   assert.equal(await a.locator('#hand-zone').isVisible(),false,'the waiting room must not show fake empty cards');assert.equal(await a.locator('#game-actions').getByRole('button',{name:'Start game',exact:true}).isDisabled(),true);await a.getByText('Waiting for 2 more players.',{exact:true}).waitFor();
   for(const page of pages.slice(1)){if(await page.locator('#room').isVisible())await page.locator('#back').click();await page.getByLabel('Table code').fill(code);await page.getByRole('button',{name:'Join table',exact:true}).click();}
   await a.getByRole('button',{name:'Start game',exact:true}).click();await a.locator('#hand.durak-hand .card').first().waitFor();
   assert.equal(await a.locator('#hand .card').count(),6);assert.equal(await a.getByRole('button',{name:'Save & acknowledge deck'}).count(),0);
   let view=await api(a,`/rooms/${code}`);assert.equal(view.table.kind,'durak');
   // Exercise the actual one-tap opening control before driving the full game over authenticated HTTP.
   const opener=pages[ids.indexOf(view.table.attacker)];await opener.waitForFunction(()=>[...document.querySelectorAll('#hand .card')].some(c=>!c.disabled));const opening=await opener.locator('#hand .card:enabled').first().getAttribute('data-card');await opener.locator('#hand .card:enabled').first().click();
   await opener.waitForFunction(card=>!document.querySelector(`#hand [data-card="${card}"]`),opening);
   view=await api(opener,`/rooms/${code}`);
   if(process.argv.includes('--ui-only'))break;
   for(let step=0;step<1500 && view.phase!=='finished';step++){
     const t=view.table,actor=t.pairs.some(p=>!p.cover)&&!t.taking?t.defender:t.turn;
     const i=ids.indexOf(actor);assert.ok(i>=0,'game must have an actor');
     view=await api(pages[i],`/rooms/${code}`);const c=view.table.controls;
     let action;
     if(c.transferCards.length)action={type:'transfer',card:c.transferCards[0]};
     else {const defence=c.defences.find(d=>d.cards.length);
       if(defence)action={type:'defend',attack:defence.attack,card:defence.cards[0]};
       else if(c.canTake)action={type:'take'};
       else if(c.attackCards.length)action={type:'attack',cards:[c.attackCards[0]]};
       else if(c.canPass)action={type:'pass'};
     }
     assert.ok(action,'game must have an action');
     view=await api(pages[i],`/rooms/${code}/commands`,{version:view.version,requestId:crypto.randomUUID(),action:{type:'durak',action}});
     // Pace the real HTTP flow below the unchanged per-IP production rate limit.
     await new Promise(resolve=>setTimeout(resolve,150));
   }
   console.log('Completed real Durak game',game);assert.equal(view.phase,'finished');const audit=await api(a,`/rooms/${code}/audit`);assert.equal(audit.deals.length,1);const path=join(directory,`${game}.json`);await writeFile(path,JSON.stringify(audit));const verifier=spawn(resolve('target/debug/verify-audit'),[path],{stdio:['ignore','pipe','inherit']});let verification='';verifier.stdout.on('data',c=>verification+=c);assert.equal((await once(verifier,'exit'))[0],0);assert.equal(JSON.parse(verification).valid,true);
   await a.reload();await a.getByRole('button',{name:'Rematch',exact:true}).waitFor();await a.getByRole('button',{name:'Rematch',exact:true}).click();await a.waitForFunction(()=>document.querySelector('#room').dataset.phase==='attack'&&document.querySelector('#deal-badge').textContent==='DEAL 02');assert.equal((await api(a,`/rooms/${code}/audit`)).decks[1].proof,undefined);
 }
 // Controlled private projections isolate UI geometry, rank requests and swipe behavior.
 const page=pages[0],current=await api(page,'/rooms/active');let fixture=structuredClone(current);fixture.version+=10000;const me=ids[0],other=ids[1],helper=ids[2];
 const all=['C','D','H','S'].flatMap(s=>['6','7','8','9','10','J','Q','K','A'].map(r=>r+s));
 fixture.table={...fixture.table,phase:'attack',hand:all,seats:fixture.players.map(p=>({...p,count:p.id===me?36:0,finished:false})),attacker:me,originalAttacker:me,defender:other,turn:me,pairs:[],taking:false,requests:[],approvals:[],stockCount:0,discardCount:0,controls:{attackCards:all,defences:[],transferCards:[],requestRanks:[],canTake:false,canPass:false}};
 await page.route(`**/api/rooms/${fixture.code}`,route=>route.fulfill({json:fixture}));
 await page.route('**/api/rooms/active',route=>route.fulfill({json:fixture}));
 async function controlClick(locator,options){const response=page.waitForResponse(r=>r.url().endsWith('/commands')&&r.request().method()==='POST');await locator.click(options);await response;await page.waitForFunction(version=>document.querySelector('#room').dataset.version===String(version),fixture.version);}
 let sent=[];await page.route(`**/api/rooms/${fixture.code}/commands`,async route=>{sent.push(route.request().postDataJSON());fixture.version++;await route.fulfill({json:fixture});});
 await page.reload();await page.waitForFunction(()=>document.querySelectorAll('#hand .card').length===36);
 for(const [width,height] of [[320,480],[390,844],[844,390],[1024,768]]){
   await page.setViewportSize({width,height});await page.waitForFunction(()=>parseFloat(document.documentElement.style.getPropertyValue('--app-height'))===innerHeight);const layout=await page.evaluate(()=>{const hand=document.getElementById('hand'),r=hand.getBoundingClientRect(),cards=[...hand.querySelectorAll('.card')].map(c=>c.getBoundingClientRect());const names=[...document.querySelectorAll('.durak-seat .seat-name')],buttons=[...document.querySelectorAll('#game-actions button')].filter(e=>e.getBoundingClientRect().height>0);return{readableCards:cards.every(c=>c.width>=76),readableNames:names.every(e=>parseFloat(getComputedStyle(e).fontSize)>=12),actionsInside:buttons.every(e=>{const r=e.getBoundingClientRect();return r.top>=0&&r.bottom<=innerHeight+1;}),readableActions:buttons.every(e=>parseFloat(getComputedStyle(e).fontSize)>=15&&e.getBoundingClientRect().height>=44),rows:new Set(cards.map(c=>Math.round(c.top))).size,scrolls:hand.scrollWidth>hand.clientWidth,overflow:document.documentElement.scrollWidth>innerWidth,handInside:r.bottom<=innerHeight,targets:cards.every(c=>c.width>=44&&c.height>=44)};});
   console.log('Durak hand geometry',width,height,layout);assert.equal(layout.rows,2);assert.equal(layout.scrolls,true);assert.equal(layout.overflow,false);assert.equal(layout.handInside,true);assert.equal(layout.targets,true);assert.equal(layout.readableCards,true);assert.equal(layout.readableNames,true);assert.equal(layout.readableActions,true);assert.equal(layout.actionsInside,true);
 }
 await page.setViewportSize({width:390,height:844});await page.waitForFunction(()=>parseFloat(document.documentElement.style.getPropertyValue('--app-height'))===innerHeight);await page.screenshot({path:'/tmp/pastimer-durak-large-hand.png',fullPage:true});await page.locator('#hand').evaluate(e=>e.scrollLeft=180);fixture.version++;await page.waitForFunction(version=>document.querySelector('#room').dataset.version===String(version),fixture.version);assert.ok(await page.locator('#hand').evaluate(e=>e.scrollLeft)>=170);
 const hand=await page.locator('#hand').boundingBox();
 const cdp=await page.context().newCDPSession(page);await page.locator('#hand').evaluate(e=>e.scrollLeft=0);
 await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:hand.x+230,y:hand.y+30,id:1}]});
 for(let i=1;i<=8;i++)await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:hand.x+230-i*20,y:hand.y+30,id:1}]});
 await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});
 await page.waitForFunction(()=>document.querySelector('#hand').scrollLeft>30);assert.equal(sent.length,0,'touch scrolling must not play a card');await cdp.detach();
 await page.mouse.move(hand.x+190,hand.y+30);await page.mouse.down();await page.mouse.move(hand.x+70,hand.y+30,{steps:8});await page.mouse.up();assert.equal(sent.length,0,'dragging must not play a card');
 fixture.table.controls.attackCards=[];fixture.version++;await page.waitForFunction(version=>document.querySelector('#room').dataset.version===String(version),fixture.version);
 await page.locator('#hand').evaluate(e=>e.scrollLeft=0);await page.mouse.move(hand.x+190,hand.y+30);await page.mouse.down();await page.mouse.move(hand.x+70,hand.y+30,{steps:8});await page.mouse.up();assert.ok(await page.locator('#hand').evaluate(e=>e.scrollLeft)>30,'a waiting player can drag a disabled hand');
 fixture.table.controls.attackCards=all;fixture.version++;await page.waitForFunction(version=>document.querySelector('#room').dataset.version===String(version),fixture.version);
 await page.locator('#hand').evaluate(e=>e.scrollLeft=e.scrollWidth);await controlClick(page.locator('#hand .card').last());assert.equal(sent.length,1,'rightmost overflow card is playable');
 for(const count of [1,3,5]){
  fixture.table.hand=all.slice(0,count);fixture.table.controls.attackCards=fixture.table.hand;fixture.version++;await page.waitForFunction(version=>document.querySelector('#room').dataset.version===String(version),fixture.version);
  const last=page.locator('#hand .card').last(),box=await last.boundingBox(),face=await last.locator('svg').boundingBox(),code=await last.getAttribute('data-card');assert.ok(box.height>=face.height-1,'an unpaired card must have a full-face tap target');
  await controlClick(last,{position:{x:box.width/2,y:box.height*.8}});assert.deepEqual(sent.at(-1).action,{type:'durak',action:{type:'attack',cards:[code]}});
 }
 fixture.table={...fixture.table,hand:['9H','JC'],attacker:other,defender:helper,turn:other,pairs:[{card:'6C',player:other,cover:'9C'}],controls:{attackCards:[],defences:[],transferCards:[],requestRanks:['9'],canTake:false,canPass:false}};fixture.version++;await page.reload();await controlClick(page.getByRole('button',{name:'I want to throw in: 9',exact:true}));assert.deepEqual(sent.at(-1).action,{type:'durak',action:{type:'request',rank:'9'}});
 fixture.table.pairs.push({card:'9D',player:other,cover:'JD'});fixture.table.approvals=[{player:me,rank:'9'}];fixture.table.controls.requestRanks=['J'];fixture.version++;await page.reload();await controlClick(page.getByRole('button',{name:'I want to throw in: J',exact:true}));assert.equal(sent.at(-1).action.action.rank,'J');
 fixture.table.attacker=me;fixture.table.requests=[{player:helper,rank:'J'}];fixture.table.controls.requestRanks=[];fixture.version++;await page.reload();await controlClick(page.getByRole('button',{name:'Allow Durak 2: J',exact:true}));assert.deepEqual(sent.at(-1).action,{type:'durak',action:{type:'approve',player:helper,rank:'J'}});
 fixture.players.find(p=>p.id===helper).name='Computer 2';fixture.version++;await page.reload();
 for(const [locale,label] of [['ru','Разрешить Компьютер 2: J'],['de','Computer 2 erlauben: J'],['es','Permitir a Ordenador 2: J'],['en','Allow Computer 2: J']]){await page.locator('#language').selectOption(locale);await page.getByRole('button',{name:label,exact:true}).waitFor();if(locale==='ru')assert.match(await page.locator('#phase').innerText(),/атакует Компьютер 2/);}
 await page.screenshot({path:'/tmp/pastimer-durak-mobile.png',fullPage:true});
 // Readability must include actual attack cards and controls, not only hand hit boxes.
 fixture.players.push(...Array.from({length:3},(_,i)=>({id:`fixture:${i}`,name:`Computer ${i+3}`})));
 fixture.table={...fixture.table,hand:['AC','AD','AH','AS','KC','KD'],attacker:other,defender:me,turn:other,requests:[],approvals:[],pairs:['6C','6D','6H','6S','7C','7D'].map((card,i)=>({card,player:other,cover:i<5?['8C','8D','8H','8S','9C'][i]:null})),seats:fixture.players.map(p=>({...p,count:6,finished:false})),controls:{attackCards:[],defences:[{attack:'7D',cards:['AD']}],transferCards:[],requestRanks:[],canTake:true,canPass:false}};fixture.version++;
 await page.reload();
 for(const [width,height] of [[320,480],[390,650],[390,844],[844,390]]){
  await page.setViewportSize({width,height});await page.waitForFunction(()=>parseFloat(document.documentElement.style.getPropertyValue('--app-height'))===innerHeight);
  const metrics=await page.evaluate(()=>{const region=document.querySelector('.durak-pairs').getBoundingClientRect(),button=document.querySelector('.durak-attack'),r=button.getBoundingClientRect();return{width:r.width,height:r.height,regionHeight:region.height,visible:r.top>=region.top-1&&r.bottom<=region.bottom+1,actionFont:parseFloat(getComputedStyle(document.querySelector('#game-actions button')).fontSize)};});
  console.log('Durak attack readability',width,height,metrics);assert.ok(metrics.width>=44&&metrics.height>=68);assert.equal(metrics.visible,true);assert.ok(metrics.actionFont>=15);
  assert.equal(await page.locator('.durak-pairs').evaluate(e=>{const b=e.querySelector('.is-target').getBoundingClientRect(),r=e.getBoundingClientRect();return b.left>=r.left-1&&b.right<=r.right+1;}),true,'keep the selected defence target visible when resizing');
  await page.screenshot({path:`/tmp/pastimer-durak-readability-${width}x${height}.png`,fullPage:true});
 }
 await page.locator('.durak-pairs').evaluate(e=>e.scrollLeft=0);fixture.version++;await page.waitForFunction(version=>document.querySelector('#room').dataset.version===String(version),fixture.version);assert.equal(await page.locator('.durak-pairs').evaluate(e=>e.scrollLeft),0,'a refresh must preserve inspection scroll when the target did not change');
 await page.getByRole('button',{name:'Defend against 7D',exact:true}).click();await controlClick(page.locator('#hand [data-card="AD"]'));assert.deepEqual(sent.at(-1).action,{type:'durak',action:{type:'defend',attack:'7D',card:'AD'}});
 assert.deepEqual(errors,[]);
 console.log(process.argv.includes('--ui-only')?'PASS: focused Durak controls and geometry':'PASS: both Durak variants, full real multiplayer games and audits, rematch/privacy, two-row 36-card hand at four sizes, scroll persistence, swipe safety, fresh-rank request and approval controls, four languages');
}catch(error){if(browser){for(const context of browser.contexts()){const page=context.pages()[0];if(page){console.error('UI FAILURE',await page.locator('body').innerText());await page.screenshot({path:'/tmp/pastimer-durak-failure.png',fullPage:true});break;}}}throw error;}finally{await browser?.close();if(server.exitCode===null){const done=once(server,'exit');server.kill('SIGTERM');await done;}await rm(directory,{recursive:true,force:true});}
