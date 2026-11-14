import {renderThousand,clearThousand,handCard,incoming,selectedTransfers,inspectTrick,bindDistributionDrag} from '/thousand.js';
import {playingCard,cardBack,sortedHand,layoutHand} from '/cards.js';
const $ = id => document.getElementById(id);
let me, room, busy = false, refreshing = false, serverOffset=0;
let sortMode=localStorage.getItem('pastimer-hand-sort')??'suit';
let pending=null;
const stored=key=>{try{return JSON.parse(localStorage.getItem(key)||'null');}catch{return null;}};
const roomKey=()=>`pastimer-room:${me.id}`;
const pendingKey=()=>`pastimer-pending:${me.id}`;
function setPending(value){pending=value;if(me){if(value)localStorage.setItem(pendingKey(),JSON.stringify(value));else localStorage.removeItem(pendingKey());}sessionStorage.removeItem('pending-command');}
function rememberRoom(code){localStorage.setItem(roomKey(),code);sessionStorage.setItem('room',code);$('resume-table').dataset.code=code;$('resume-table').textContent=`Return to table ${code}`;}
function repaint(){if(room)render(room);}
function viewport(){const tg=window.Telegram?.WebApp;document.documentElement.style.setProperty('--app-height',`${Math.min(innerHeight,tg?.viewportHeight||innerHeight,visualViewport?.height||innerHeight)}px`);layoutHand();}
async function api(path, body) {
  const response = await fetch(`/api${path}`, { ...(body === undefined ? {} : {method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(body)}) });
  const value = await response.json();
  if (!response.ok) { const error = new Error(value.error || 'Request failed'); error.status = response.status; throw error; }
  return value;
}
function notice(message = '') { $('notice').textContent = message; }
function element(tag, text, className) { const e = document.createElement(tag); e.textContent = text; if(className) e.className = className; return e; }
function button(text, handler, className) { const e = element('button',text,className); e.onclick = () => run(handler); return e; }
async function run(fn) { if(busy)return; busy=true; try { notice(); await fn(); } catch(e) { notice(e.message); } finally { busy=false; } }
function lobby(){if(room)rememberRoom(room.code);document.body.classList.remove('in-room');clearThousand();document.querySelector('.intro').hidden=false;room=undefined;$('room').hidden=true;$('lobby').hidden=false;$('resume-table').hidden=!localStorage.getItem(roomKey());try{window.Telegram?.WebApp?.BackButton?.hide?.();window.Telegram?.WebApp?.disableClosingConfirmation?.();}catch{} }
function signedIn(player){me=player;const old=stored(pendingKey())??(()=>{try{return JSON.parse(sessionStorage.getItem('pending-command')||'null');}catch{return null;}})();if(old?.actor===me.id)setPending(old);$('identity').textContent=player.name;$('login').hidden=true;$('lobby').hidden=false;}
async function command(action) {
  if(pending && (pending.code!==room.code || JSON.stringify(pending.action)!==JSON.stringify(action))) throw new Error('Retry the previous move before sending a different one.');
  if(!pending) setPending({actor:me.id,code:room.code,version:room.version,requestId:crypto.randomUUID(),action});
  try {
    let next;
    try { next = await api(`/rooms/${pending.code}/commands`,pending); }
    catch(e) {
      if(e.status!==409 || e.message!=='State changed; refresh and retry') throw e;
      // A definite version rejection means that command did not commit. Refresh and retry once.
      const latest=await api(`/rooms/${pending.code}`);
      setPending({...pending,version:latest.version,requestId:crypto.randomUUID()});
      next=await api(`/rooms/${pending.code}/commands`,pending);
    }
    setPending(null); render(next);
  } catch(e) {
    if(e.status && e.status<500) setPending(null); // A network/server failure keeps the exact command.
    render(await api(`/rooms/${room.code}`));
    throw e;
  }
}
function renderSeats(){
  const own=room.players.findIndex(p=>p.id===me.id),others=Array.from({length:room.players.length-1},(_,i)=>room.players[(own+i+1)%room.players.length]);
  const inactive=room.players.length===4?(room.table?.phase==='awaitShuffle'&&room.table.lastDeal?room.table.lastDeal.dealer:room.table?.dealer):null;
  const activeOthers=others.filter(p=>p.id!==inactive);
  const colors=['#746276','#7b654c','#416b72','#596d59'];
  $('seats').replaceChildren(...room.players.map((p,index)=>{
    const seat=room.table?.seats?.find(s=>s.id===p.id),isMe=p.id===me.id,isTurn=room.table?.turn===p.id,pile=room.table?.piles?.find(v=>v.player===p.id);
    let position=isMe?'seat-self':activeOthers.indexOf(p)===0?'seat-left':activeOthers.indexOf(p)===1?'seat-right':'seat-self';
    if(p.id===inactive)position='seat-dealer';
    const box=element('div','',`seat ${position}${isTurn?' is-turn':''}${pile?.top?' has-trick':''}`);box.dataset.player=p.id;if(pile?.top){box.title=`Inspect the last trick won by ${p.name}`;box.onclick=e=>{if(!box.dataset.recipient&&!e.target.closest('.seat-hand'))inspectTrick(room,p.id);};}
    const avatar=element('span',room.computers?.includes(p.id)?'C':p.name.trim().slice(0,1).toUpperCase(),'avatar');avatar.style.setProperty('--avatar-color',colors[index%colors.length]);
    const info=element('div','','seat-info');info.append(element('div',p.name+(isMe?' - you':''),'seat-name'));
    const score=element('div','','seat-score');score.append(element('strong',String(seat?.score??room.scores[p.id]??0)),element('span','pts'));
    if(seat){const hand=element('button','','seat-hand');hand.type='button';hand.disabled=!pile?.top;hand.setAttribute('aria-label',`Last trick won by ${p.name}`);hand.dataset.pilePlayer=p.id;hand.append(cardBack(),element('span',String(seat.count)));if(pile?.top)hand.append(element('span','↗','pile-available'));hand.onclick=()=>inspectTrick(room,p.id);score.append(hand);}
    if(seat?.sticks)score.append(element('span',`${seat.sticks} sticks`,'stick-marks'));
    info.append(score);const role=seat?.onBarrel?'BARREL':seat?.role==='dealer'?'DEALER':seat?.role==='contractor'?`CONTRACT ${room.table.order}`:isTurn?'TURN':'';if(role)info.append(element('span',role,'seat-role'));box.append(avatar,info);return box;
  }));
}
function render(next) {
  if(room?.code===next.code && next.version<room.version)next=room;
  if(next!==room&&next.table)serverOffset=next.table.serverNow-Date.now();
  room=next;document.body.classList.add('in-room');
  $('room').dataset.phase=room.table?.phase??room.phase;$('room').dataset.version=String(room.version);$('room').dataset.game=room.game;
  document.querySelector('.intro').hidden=true;rememberRoom(room.code);$('resume-table').hidden=true;try{window.Telegram?.WebApp?.enableClosingConfirmation?.();window.Telegram?.WebApp?.BackButton?.show?.();}catch{}$('room').dataset.playerCount=String(room.players.length);$('lobby').hidden=true;$('room').hidden=false;
  const dealer=room.table?.seats?.find(s=>s.id===room.table.dealer);$('game-title').textContent=dealer?`Dealer: ${dealer.name}${room.players.length===4?` - ${dealer.score} pts`:''}`:'PRIVATE TABLE';$('game-title').title=$('game-title').textContent;$('room-code').textContent=room.code;
  $('deal-badge').textContent=room.table?.deal?`DEAL ${String(room.table.deal).padStart(2,'0')}`:`${room.players.length} / ${room.options?.playerCount??4} SEATS`;
  renderSeats();clearThousand();$('table-actions').replaceChildren();$('game-actions').replaceChildren();$('thousand-table').replaceChildren();
  $('phase').textContent={lobby:'Invite your friends. Share the table code.',committing:'The deck is committed. Confirm to start the demo.',playing:`Round ${room.table?.round??''} · choose your card.`,finished:'Game complete. The record is ready to verify.'}[room.phase];
  if(pending&&pending.code===room.code)$('table-actions').append(button('Retry previous move',()=>command(pending.action),'secondary'));
  if(room.phase==='lobby'&&room.host===me.id){const start=button('Start game',()=>command({type:'start'}));start.disabled=room.players.length<(room.options?.playerCount??2);$('table-actions').append(start);}
  if(room.phase==='committing'&&!room.deck.acknowledgements.includes(me.id))$('table-actions').append(button('Save & acknowledge deck',async()=>{saveReceipt();await command({type:'acknowledge',shuffleId:room.deck.shuffleId,commitment:room.deck.commitment});}));
  const gifts=room.table?.kind==='thousand'?incoming(room,me):[],held=new Set(gifts.map(g=>g.card));
  const chosen=room.table?.kind==='thousand'?selectedTransfers(room):[];
  const hand=sortedHand((room.table?.hand??[]).filter(c=>!held.has(c)&&!chosen.includes(c)),sortMode,room.game);
  $('hand-zone').hidden=hand.length===0&&gifts.length===0;$('hand-count').textContent=`${(room.table?.hand?.length??0)-chosen.length} cards`;
  $('room').dataset.receiving=String(gifts.length>0);$('sort-hand').hidden=gifts.length>0;
  $('sort-hand').textContent=`Sort: ${sortMode}`;$('sort-hand').setAttribute('aria-label',`Sort hand by ${sortMode==='suit'?'rank':'suit'}`);
  $('incoming-zone').replaceChildren();
  $('hand').replaceChildren(...hand.map(code=>{const card=playingCard(code,{interactive:true,selected:chosen.includes(code)});card.onclick=()=>{if(room.table?.kind==='thousand')handCard(code,{room,me,send:action=>command({type:'thousand',action}),run,repaint});else run(()=>command({type:'play',card:code}));};return card;}));
  $('audit').disabled=false;$('audit').textContent='Download audit record';
  if(room.phase==='playing')for(const action of room.table?.actions??[]){if(action.input==='card')continue;$('game-actions').append(button(action.label,()=>command({type:action.type})));}
  $('commitment').textContent=room.deck?`Shuffle ${room.deck.shuffleId}\nSHA-256 ${room.deck.commitment}`:'The deck hash will appear before dealing.';
  if(room.table?.kind==='thousand'){
    renderThousand({room,me,send:action=>command({type:'thousand',action}),refresh:refreshRoom,sendPlatform:action=>command(action),run,repaint,now:()=>Date.now()+serverOffset});
    for(const card of $('hand').children){card.disabled=gifts.length>0||(!room.table.controls.recipients.length&&!room.table.controls.legalCards.includes(card.dataset.card));card.classList.toggle('is-unplayable',room.table.turn===me.id&&card.disabled);}
    bindDistributionDrag({room,repaint});
    $('commitment').textContent=room.deck?`Deal ${room.table.deal} · SHA-256 ${room.deck.commitment}`:'The dealer will publish the deck hash before dealing.';
  }else{
    for(const card of $('hand').children)card.disabled=!(room.table?.actions??[]).some(a=>a.input==='card');
    $('thousand-table').hidden=false;const center=element('div','','table-center');const logo=element('div','♠','bid-emblem');logo.style.cssText='font:45px Georgia;color:#d8c08d';center.append(logo,element('p',room.phase==='lobby'?`${room.players.length} friends at the table`:'The next hand awaits.','table-instruction'));$('thousand-table').append(center);
  }
  layoutHand();saveReceipt();
}
function saveReceipt() {
  // Local receipts provide an anchor outside the server database. Export to preserve across devices.
  const key=`pastimer-receipts:${room.id}`;
  const receipts=JSON.parse(localStorage.getItem(key)||'[]');
  if(!receipts.some(r=>r.version===room.version)) receipts.push({roomId:room.id,version:room.version,eventHash:room.latestEventHash,deck:room.deck,observedAt:new Date().toISOString()});
  localStorage.setItem(key,JSON.stringify(receipts));
  $('receipt').textContent=`${receipts.length} receipts saved in this browser.`;
}
$('login-form').onsubmit=e=>{e.preventDefault();run(async()=>signedIn(await api('/auth',{name:new FormData(e.target).get('name')})));};
$('create-form').onsubmit=e=>{e.preventDefault();run(async()=>{const data=new FormData(e.target),game=data.get('game');render(await api('/rooms',{game,...(game==='thousand'?{options:{playerCount:Number(data.get('playerCount')),barrelThreshold:Number(data.get('barrelThreshold'))},computerCount:Number(data.get('computerCount'))}:{})}));});};
function configureComputers(){
  const seats=Number(document.querySelector('[name=playerCount]').value),select=$('computer-count');
  const count=Math.min(Number(select.value)||0,seats-1);
  select.replaceChildren(...Array.from({length:seats},(_,i)=>{const option=element('option',i?`${i} computer${i===1?'':'s'} - ${seats-i===1?'solo':`${seats-i} people`}`:'No computers');option.value=String(i);return option;}));
  select.value=String(count);const humans=seats-count;
  $('computer-note').textContent=humans===1?'Just you and the computers. Start whenever you are ready.':`Invite ${humans-1} friend${humans===2?'':'s'} to fill the table${count?' - computers take the remaining seats':''}.`;
}
document.querySelector('[name=playerCount]').onchange=configureComputers;
$('computer-count').onchange=configureComputers;
$('games').onchange=()=>{$('thousand-settings').hidden=$('games').value!=='thousand';configureComputers();};
$('join-form').onsubmit=e=>{e.preventDefault();run(async()=>render(await api(`/rooms/${String(new FormData(e.target).get('code')).trim().toUpperCase()}/join`,{})));};
$('back').onclick=lobby;
$('sort-hand').onclick=()=>{sortMode=sortMode==='suit'?'rank':'suit';localStorage.setItem('pastimer-hand-sort',sortMode);if(room)render(room);};
$('copy-code').onclick=()=>run(async()=>{if(navigator.clipboard){await navigator.clipboard.writeText(room.code);notice('Table code copied. Send it to your friends.');}else notice(`Your table code is ${room.code}`);});
window.addEventListener('resize',viewport);visualViewport?.addEventListener('resize',viewport);window.Telegram?.WebApp?.onEvent?.('viewportChanged',viewport);window.Telegram?.WebApp?.BackButton?.onClick?.(lobby);viewport();
$('resume-table').onclick=()=>run(async()=>render(await api(`/rooms/${$('resume-table').dataset.code}`)));
$('show-record').onclick=()=>$('record-dialog').showModal();
for(const button of document.querySelectorAll('[data-close]'))button.onclick=()=>button.closest('dialog').close();
$('audit').onclick=()=>run(async()=>{ const audit=await api(`/rooms/${room.code}/audit`); const receipts=JSON.parse(localStorage.getItem(`pastimer-receipts:${room.id}`)||'[]');const url=URL.createObjectURL(new Blob([JSON.stringify({...audit,receipts},null,2)],{type:'application/json'}));const a=element('a','');a.href=url;a.download=`pastimer-${room.code}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000); });
await run(async()=>{
  const games=await api('/games'); $('games').replaceChildren(...games.map(g=>{const e=element('option',g.name);e.value=g.id;return e;})); $('games').value='thousand'; $('games').onchange();
  try { signedIn(await api('/me')); } catch {
    const tg=window.Telegram?.WebApp;
    if(tg?.initData){tg.ready();tg.expand();tg.setHeaderColor?.('#101e28');tg.setBackgroundColor?.('#101e28');signedIn(await api('/auth',{initData:tg.initData}));}
    else {$('login').hidden=false;return;}
  }
  const previous=localStorage.getItem(roomKey())??sessionStorage.getItem('room');
  if(previous){try{render(await api(`/rooms/${previous}`));return;}catch(error){if(![403,404,400].includes(error.status))throw error;localStorage.removeItem(roomKey());}}
  const active=await api('/rooms/active');if(active)render(active);
});
async function refreshRoom(){
  if(!room||busy||refreshing)return;
  refreshing=true;
  try{const code=room.code;const next=await api(`/rooms/${code}`);if(room?.code===code&&(next.version>room.version||next.computerDueAt!==room.computerDueAt))render(next);}
  catch(e){notice(e.message);}
  finally{refreshing=false;}
}
setInterval(refreshRoom,500);
