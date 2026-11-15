import {renderThousand,clearThousand,handCard,incoming,selectedTransfers,inspectTrick,bindDistributionDrag,layoutTrick} from '/thousand.js';
import {installLanguages} from '/i18n.js';
import {installArchive} from '/archive.js';
import {playingCard,cardBack,sortedHand,layoutHand} from '/cards.js';
const $ = id => document.getElementById(id);
let me, room, busy = false, refreshing = false, serverOffset=0;
let sortMode=localStorage.getItem('pastimer-hand-sort')??'suit';
let pending=null,handSlotKey='',handSlots=[],forcedMove=null;
const archive=installArchive({api,run,openTable:async code=>{await api(`/rooms/${code}/join`,{});render(await api(`/rooms/${code}`));}});
const stored=key=>{try{return JSON.parse(localStorage.getItem(key)||'null');}catch{return null;}};
const roomKey=()=>`pastimer-room:${me.id}`;
const pendingKey=()=>`pastimer-pending:${me.id}`;
function setPending(value){pending=value;if(me){if(value)localStorage.setItem(pendingKey(),JSON.stringify(value));else localStorage.removeItem(pendingKey());}sessionStorage.removeItem('pending-command');}
function rememberRoom(code){localStorage.setItem(roomKey(),code);sessionStorage.setItem('room',code);$('resume-table').dataset.code=code;$('resume-table').textContent=`Return to table ${code}`;}
function repaint(){if(room)render(room);}
function viewport(){const tg=window.Telegram?.WebApp;document.documentElement.style.setProperty('--app-height',`${Math.min(innerHeight,tg?.viewportHeight||innerHeight,visualViewport?.height||innerHeight)}px`);layoutHand();layoutTrick();}
async function api(path, body) {
  const response = await fetch(`/api${path}`, { ...(body === undefined ? {} : {method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(body)}) });
  const value = await response.json();
  if (!response.ok) { const error = new Error(value.error || 'Request failed'); error.status = response.status; throw error; }
  return value;
}
function notice(message = '') { $('notice').textContent = message; }
function element(tag, text, className) { const e = document.createElement(tag); e.textContent = text; if(className) e.className = className; return e; }
function button(text, handler, className) { const e = element('button',text,className); e.onclick = () => run(handler); return e; }
async function run(fn) { if(busy)return; busy=true; try { notice(); await fn(); } catch(e) { notice(e.message); } finally { busy=false; syncForcedMove(); } }
function lobby(){cancelForcedMove();if(room)rememberRoom(room.code);document.body.classList.remove('in-room');clearThousand();document.querySelector('.intro').hidden=false;room=undefined;$('room').hidden=true;$('lobby').hidden=false;$('lobby-tools').hidden=false;$('resume-table').hidden=!localStorage.getItem(roomKey());try{window.Telegram?.WebApp?.BackButton?.hide?.();window.Telegram?.WebApp?.disableClosingConfirmation?.();}catch{} }
function signedIn(player){me=player;const old=stored(pendingKey())??(()=>{try{return JSON.parse(sessionStorage.getItem('pending-command')||'null');}catch{return null;}})();if(old?.actor===me.id)setPending(old);$('identity').textContent=player.name;$('login').hidden=true;$('lobby').hidden=false;$('lobby-tools').hidden=false;}
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
    const box=element('div','',`seat ${position}${isTurn?' is-turn':''}${pile?.top?' has-trick':''}${room.table?.trickLeader===p.id?' is-leader':''}${room.table?.trickWinner===p.id?' is-taking':''}`);box.dataset.player=p.id;if(pile?.top){box.title=`Inspect the last trick won by ${p.name}`;box.onclick=e=>{if(!box.dataset.recipient&&!e.target.closest('.seat-hand'))inspectTrick(room,p.id);};}
    const avatar=element('span',room.computers?.includes(p.id)?'C':p.name.trim().slice(0,1).toUpperCase(),'avatar');avatar.style.setProperty('--avatar-color',colors[index%colors.length]);
    const info=element('div','','seat-info');const name=element('div','','seat-name'),barrel=element('span','','barrel-marker');barrel.setAttribute('role','img');barrel.setAttribute('aria-label','On the barrel');barrel.style.visibility=seat?.onBarrel?'visible':'hidden';barrel.innerHTML='<svg viewBox="0 0 20 26" aria-hidden="true"><path d="M5 3Q0 13 5 23Q10 27 15 23Q20 13 15 3Z" fill="#936638" stroke="#e5bc73" stroke-width="1.6"/><path d="M3 7H17M2 18H18M8 3Q5 13 8 24M12 3Q15 13 12 24" fill="none" stroke="#e5bc73" stroke-width="1.4"/><ellipse cx="10" cy="3" rx="5" ry="2" fill="#bd8b4e" stroke="#e5bc73"/></svg>';name.append(element('span',p.name+(isMe?' - you':''),'player-name'),barrel);info.append(name);
    const score=element('div','','seat-score');const inDeal=room.table?.kind==='thousand'&&!['awaitShuffle','finished'].includes(room.table.phase);score.title=inDeal?`Deal points: ${seat?.roundPoints??0}; match score: ${seat?.score??0}`:'Match score';score.append(element('strong',String(inDeal?(seat?.roundPoints??0):(seat?.score??room.scores[p.id]??0))),element('span','pts','score-kind'));
    if(seat){const hand=element('button','','seat-hand');hand.type='button';hand.disabled=!pile?.top;hand.setAttribute('aria-label',`Last trick won by ${p.name}`);hand.dataset.pilePlayer=p.id;hand.append(cardBack(),element('span',String(seat.count)));if(pile?.top)hand.append(element('span','↗','pile-available'));hand.onclick=()=>inspectTrick(room,p.id);score.append(hand);}
    if(seat?.sticks)score.append(element('span','Ⅰ'.repeat(seat.sticks),'stick-marks'));
    info.append(score);const contract=room.table?.contractor===p.id?`CONTRACT ${room.table.order}`:'',role=contract||(seat?.role==='dealer'?'DEALER':isTurn&&!room.table?.trickLeader?'TURN':'');info.append(element('span',role,'seat-role'));box.append(avatar,info);if(room.table?.trickWinner===p.id)box.append(element('span','TAKES TRICK','seat-activity'));else if(room.table?.trickLeader===p.id)box.append(element('span','LEAD','seat-activity'));return box;
  }));
}
function render(next) {
  if(room?.code===next.code && next.version<room.version)next=room;
  if(next!==room&&next.table)serverOffset=next.table.serverNow-Date.now();
  room=next;document.body.classList.add('in-room');
  $('room').dataset.phase=room.table?.phase??room.phase;$('room').dataset.version=String(room.version);$('room').dataset.game=room.game;
  document.querySelector('.intro').hidden=true;rememberRoom(room.code);$('resume-table').hidden=true;try{window.Telegram?.WebApp?.enableClosingConfirmation?.();window.Telegram?.WebApp?.BackButton?.show?.();}catch{}$('room').dataset.playerCount=String(room.players.length);$('lobby').hidden=true;$('lobby-tools').hidden=true;$('room').hidden=false;
  const dealer=room.table?.seats?.find(s=>s.id===room.table.dealer);$('game-title').textContent=dealer?`Dealer: ${dealer.name}`:'PRIVATE TABLE';$('game-title').title=$('game-title').textContent;$('room-code').textContent=room.code;
  $('deal-badge').textContent=room.table?.deal?`DEAL ${String(room.table.deal).padStart(2,'0')}`:`${room.players.length} / ${room.options?.playerCount??4} SEATS`;
  renderSeats();clearThousand();$('table-actions').replaceChildren();$('game-actions').replaceChildren();$('thousand-table').replaceChildren();
  $('phase').textContent={lobby:'Invite your friends. Share the table code.',committing:'The deck is committed. Confirm to start the demo.',playing:`Round ${room.table?.round??''} · choose your card.`,finished:'Game complete. The record is ready to verify.'}[room.phase];
  if(pending&&pending.code===room.code)$('table-actions').append(button('Retry previous move',()=>command(pending.action),'secondary'));
  if(room.phase==='lobby'&&room.host===me.id){const start=button('Start game',()=>command({type:'start'}));start.disabled=room.players.length<(room.options?.playerCount??2);$('table-actions').append(start);}
  if(room.phase==='committing'&&!room.deck.acknowledgements.includes(me.id))$('table-actions').append(button('Save & acknowledge deck',async()=>{saveReceipt();await command({type:'acknowledge',shuffleId:room.deck.shuffleId,commitment:room.deck.commitment});}));
  const gifts=room.table?.kind==='thousand'?incoming(room,me):[],held=new Set(gifts.map(g=>g.card));
  const chosen=room.table?.kind==='thousand'?selectedTransfers(room):[];
  const hand=sortedHand((room.table?.hand??[]).filter(c=>!held.has(c)&&!chosen.includes(c)),sortMode,room.game);
  $('hand-zone').hidden=false;$('hand-count').textContent=`${(room.table?.hand?.length??0)-chosen.length} cards`;
  $('room').dataset.receiving=String(gifts.length>0);$('sort-hand').style.visibility=gifts.length?'hidden':'visible';
  $('sort-hand').textContent=`Sort: ${sortMode}`;$('sort-hand').setAttribute('aria-label',`Sort hand by ${sortMode==='suit'?'rank':'suit'}`);
  $('incoming-zone').replaceChildren();
  const slotKey=`${room.id}:${room.table?.deal??0}:${sortMode}`;
  if(slotKey!==handSlotKey||hand.some(code=>!handSlots.includes(code))){handSlotKey=slotKey;handSlots=[...hand,...Array(Math.max(0,10-hand.length)).fill(null)];}
  $('hand').replaceChildren(...Array.from({length:10},(_,i)=>{const slot=element('div','','hand-slot'),code=handSlots[i];slot.dataset.slot=String(i);if(code&&hand.includes(code)){const card=playingCard(code,{interactive:true});card.onclick=()=>{if(room.table?.kind==='thousand')handCard(code,{room,me,send:action=>command({type:'thousand',action}),run,repaint});else run(()=>command({type:'play',card:code}));};slot.append(card);}else{const placeholder=element('span','','card-placeholder');placeholder.setAttribute('aria-hidden','true');slot.append(placeholder);}return slot;}));

  if(room.phase==='playing')for(const action of room.table?.actions??[]){if(action.input==='card')continue;$('game-actions').append(button(action.label,()=>command({type:action.type})));}
  if(room.table?.kind==='thousand'){
    renderThousand({room,me,send:action=>command({type:'thousand',action}),refresh:refreshRoom,sendPlatform:action=>command(action),run,repaint,now:()=>Date.now()+serverOffset});
    for(const card of $('hand').querySelectorAll('.card')){card.disabled=gifts.length>0||(!room.table.controls.recipients.length&&!room.table.controls.legalCards.includes(card.dataset.card));card.classList.toggle('is-unplayable',room.table.phase==='trick'&&room.table.turn===me.id&&card.disabled);card.classList.toggle('is-playable',room.table.controls.legalCards.includes(card.dataset.card)&&!card.disabled);}
    bindDistributionDrag({room,repaint});
    if(room.table.phase==='finished'){const again=button('New table',lobby);const history=button('Played games',()=>archive.showArchive(),'secondary');$('game-actions').replaceChildren(again,history);}
  }else{
    for(const card of $('hand').querySelectorAll('.card'))card.disabled=!(room.table?.actions??[]).some(a=>a.input==='card');
    $('thousand-table').hidden=false;const center=element('div','','table-center');const logo=element('div','♠','bid-emblem');logo.style.cssText='font:45px Georgia;color:#d8c08d';center.append(logo,element('p',room.phase==='lobby'?`Players at table: ${room.players.length}`:'The next hand awaits.','table-instruction'));$('thousand-table').append(center);
  }
  layoutHand();layoutTrick();saveReceipt();syncForcedMove();
}
function cancelForcedMove(){if(forcedMove)clearTimeout(forcedMove.timer);forcedMove=null;}
function forcedChoice(){
  const t=room?.table;if(!t||t.kind!=='thousand'||t.phase!=='trick'||t.turn!==me?.id||t.controls.legalCards.length!==1||pending||$('room').hidden||document.visibilityState!=='visible'||document.querySelector('dialog[open]')||incoming(room,me).length)return null;
  const card=t.controls.legalCards[0];return {key:`${room.id}:${room.version}:${card}`,card};
}
function syncForcedMove(){
  const choice=forcedChoice();if(!choice){cancelForcedMove();return;}
  if(forcedMove?.key!==choice.key){cancelForcedMove();forcedMove={...choice,due:performance.now()+2000,timer:null,fired:false};}
  const move=forcedMove,card=$('hand').querySelector(`[data-card="${move.card}"]`);card?.classList.add('is-forced');
  if(move.fired||move.timer||busy)return;
  $('phase').textContent='Only legal card - playing in 2 seconds.';
  move.timer=setTimeout(()=>{move.timer=null;if(busy)return;if(forcedChoice()?.key!==move.key){syncForcedMove();return;}move.fired=true;run(async()=>{
    const latest=await api(`/rooms/${room.code}`);render(latest);
    if(forcedChoice()?.key===move.key){const handCardElement=$('hand').querySelector(`[data-card="${move.card}"]`);if(handCardElement&&!handCardElement.disabled)await handCard(move.card,{room,me,send:action=>command({type:'thousand',action}),run:fn=>fn(),repaint});}
  });},Math.max(0,move.due-performance.now()));
}
function saveReceipt() {
  // Local receipts provide an anchor outside the server database. Export to preserve across devices.
  const key=`pastimer-receipts:${room.id}`;
  const receipts=JSON.parse(localStorage.getItem(key)||'[]');
  if(!receipts.some(r=>r.version===room.version)) receipts.push({roomId:room.id,version:room.version,eventHash:room.latestEventHash,deck:room.deck,observedAt:new Date().toISOString()});
  localStorage.setItem(key,JSON.stringify(receipts));
}
$('login-form').onsubmit=e=>{e.preventDefault();run(async()=>signedIn(await api('/auth',{name:new FormData(e.target).get('name')})));};
$('create-form').onsubmit=e=>{e.preventDefault();run(async()=>{const data=new FormData(e.target),game=data.get('game');render(await api('/rooms',{game,...(game==='thousand'?{options:{playerCount:Number(data.get('playerCount')),barrelThreshold:Number(data.get('barrelThreshold'))},computerCount:Number(data.get('computerCount'))}:{})}));});};
function configureComputers(){
  const seats=Number(document.querySelector('[name=playerCount]').value),select=$('computer-count');
  const count=Math.min(Number(select.value)||0,seats-1);
  select.replaceChildren(...Array.from({length:4},(_,i)=>{const option=element('option',i?`${i} computer${i===1?'':'s'} - ${i===3?'solo, 4 seats':seats-i===1?'solo':`${seats-i} people`}`:'No computers');option.value=String(i);return option;}));
  select.value=String(count);for(const b of document.querySelectorAll('[data-seats]'))b.setAttribute('aria-pressed',String(Number(b.dataset.seats)===seats));const humans=seats-count;
  $('computer-note').textContent=humans===1?'Just you and the computers. Start whenever you are ready.':`Invite ${humans-1} friend${humans===2?'':'s'} to fill the table${count?' - computers take the remaining seats':''}.`;
}
document.querySelector('[name=playerCount]').onchange=configureComputers;
$('computer-count').onchange=()=>{const n=Number($('computer-count').value),seats=document.querySelector('[name=playerCount]');if(n>=Number(seats.value))seats.value=String(n+1);configureComputers();};
for(const b of document.querySelectorAll('[data-seats]'))b.onclick=()=>{document.querySelector('[name=playerCount]').value=b.dataset.seats;configureComputers();};
$('games').onchange=()=>{$('thousand-settings').hidden=$('games').value!=='thousand';configureComputers();};
$('join-form').onsubmit=e=>{e.preventDefault();run(async()=>render(await api(`/rooms/${String(new FormData(e.target).get('code')).trim().toUpperCase()}/join`,{})));};
$('back').onclick=lobby;
$('sort-hand').onclick=()=>{sortMode=sortMode==='suit'?'rank':'suit';localStorage.setItem('pastimer-hand-sort',sortMode);if(room)render(room);};
$('copy-code').onclick=()=>run(async()=>{if(navigator.clipboard){await navigator.clipboard.writeText(room.code);notice('Table code copied. Send it to your friends.');}else notice(`Your table code is ${room.code}`);});
document.addEventListener('visibilitychange',syncForcedMove);
new MutationObserver(syncForcedMove).observe(document.body,{subtree:true,attributes:true,attributeFilter:['open']});
window.addEventListener('resize',viewport);visualViewport?.addEventListener('resize',viewport);window.Telegram?.WebApp?.onEvent?.('viewportChanged',viewport);window.Telegram?.WebApp?.BackButton?.onClick?.(lobby);viewport();
$('resume-table').onclick=()=>run(async()=>render(await api(`/rooms/${$('resume-table').dataset.code}`)));
installLanguages(()=>{repaint();archive.refreshLanguage();});
$('marriage-options').onclick=()=>$('marriage-dialog').showModal();
$('show-record').onclick=()=>run(()=>archive.showRecord(room.code));
$('open-archive').onclick=()=>run(()=>archive.showArchive());
function solo(seats){run(async()=>{if(pending)throw new Error('Return to your table and retry the previous move first.');$('games').value='thousand';document.querySelector('[name=playerCount]').value=String(seats);configureComputers();$('computer-count').value=String(seats-1);configureComputers();if(!$('create-form').reportValidity())return;render(await api('/rooms',{game:'thousand',options:{playerCount:seats,barrelThreshold:Number(document.querySelector('[name=barrelThreshold]').value)},computerCount:seats-1}));await command({type:'start'});await command(room.table.dealer===me.id?{type:'thousand',action:{type:'shuffle'}}:{type:'computerDeal'});});}
$('solo-four').onclick=()=>solo(4);$('solo-three').onclick=()=>solo(3);
for(const button of document.querySelectorAll('[data-close]'))button.onclick=()=>button.closest('dialog').close();
await run(async()=>{
  const games=await api('/games'); $('games').replaceChildren(...games.map(g=>{const e=element('option',g.id==='thousand'?'Thousand':'High card · platform demo');e.value=g.id;return e;})); $('games').value='thousand'; $('games').onchange();
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
