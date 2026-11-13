import {renderThousand,clearThousand} from '/thousand.js';
import {playingCard,cardBack,sortedHand,layoutHand} from '/cards.js';
const $ = id => document.getElementById(id);
let me, room, selected, busy = false;
let sortMode=localStorage.getItem('pastimer-hand-sort')??'suit';
let pending = JSON.parse(sessionStorage.getItem('pending-command') || 'null');
function setPending(value) { pending=value; if(value) sessionStorage.setItem('pending-command',JSON.stringify(value)); else sessionStorage.removeItem('pending-command'); }
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
function lobby() { document.body.classList.remove('in-room'); clearThousand(); document.querySelector('.intro').hidden=false; room = undefined; sessionStorage.removeItem('room'); $('room').hidden = true; $('lobby').hidden = false; }
function signedIn(player) { me = player; if(pending && pending.actor!==me.id) setPending(null); $('identity').textContent = player.name; $('login').hidden = true; $('lobby').hidden = false; }
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
    setPending(null); selected=undefined; render(next);
  } catch(e) {
    if(e.status && e.status<500) setPending(null); // A network/server failure keeps the exact command.
    render(await api(`/rooms/${room.code}`));
    throw e;
  }
}
function renderSeats(){
  const own=room.players.findIndex(p=>p.id===me.id);
  const others=Array.from({length:room.players.length-1},(_,i)=>room.players[(own+i+1)%room.players.length]);
  const activeOthers=others.filter(p=>room.table?.seats?.find(s=>s.id===p.id)?.role!=='dealer');
  const colors=['#746276','#7b654c','#416b72','#596d59'];
  $('seats').replaceChildren(...room.players.map((p,index)=>{
    const seat=room.table?.seats?.find(s=>s.id===p.id),isMe=p.id===me.id,isTurn=room.table?.turn===p.id;
    let position='seat-self';
    if(!isMe){if(seat?.role==='dealer'||activeOthers.indexOf(p)>1)position='seat-dealer';else if(activeOthers.length===1)position='seat-center';else position=activeOthers.indexOf(p)===0?'seat-left':'seat-right';}
    const box=element('div','',`seat ${position}${isTurn?' is-turn':''}`);box.dataset.player=p.id;
    const avatar=element('span',p.name.trim().slice(0,1).toUpperCase(),'avatar');avatar.style.setProperty('--avatar-color',colors[index%colors.length]);
    const info=element('div','','seat-info');info.append(element('div',p.name+(isMe?' · you':''),'seat-name'));
    const score=element('div','','seat-score');score.append(element('strong',String(seat?.score??room.scores[p.id]??0)),element('span','pts'));
    if(seat?.count){const backs=element('span','','seat-cards');if(!isMe)for(let i=0;i<Math.min(3,seat.count);i++)backs.append(cardBack());backs.append(element('span',String(seat.count)));score.append(backs);}
    if(seat?.sticks){const sticks=element('span','','stick-marks');sticks.title=`${seat.sticks} sticks`;sticks.setAttribute('aria-label',sticks.title);for(let i=0;i<(seat.sticks%3||3);i++)sticks.append(element('i',''));score.append(sticks);}
    info.append(score);
    const role=seat?.onBarrel?'ON THE BARREL':seat?.role==='dealer'?'DEALER':seat?.role==='contractor'?`CONTRACT · ${room.table.order}`:isTurn?(isMe?'YOUR TURN':'PLAYING'):room.phase==='lobby'?'READY':'';
    if(role)info.append(element('span',role,'seat-role'));box.append(avatar,info);return box;
  }));
}
function render(next) {
  if(room?.table?.deal!==next.table?.deal||!next.table?.hand?.includes(selected))selected=undefined;
  room=next;document.body.classList.add('in-room');
  $('room').dataset.phase=room.table?.phase??room.phase;$('room').dataset.version=String(room.version);$('room').dataset.game=room.game;
  document.querySelector('.intro').hidden=true;sessionStorage.setItem('room',room.code);$('lobby').hidden=true;$('room').hidden=false;
  $('game-title').textContent=room.game==='thousand'?'ТЫСЯЧА · PRIVATE TABLE':'PASTIMER · PRIVATE TABLE';$('room-code').textContent=room.code;
  $('deal-badge').textContent=room.table?.deal?`DEAL ${String(room.table.deal).padStart(2,'0')}`:`${room.players.length} / ${room.options?.playerCount??4} SEATS`;
  renderSeats();clearThousand();$('table-actions').replaceChildren();$('game-actions').replaceChildren();$('thousand-table').replaceChildren();
  $('phase').textContent={lobby:'Invite your friends. Share the table code.',committing:'The deck is committed. Confirm to start the demo.',playing:`Round ${room.table?.round??''} · choose your card.`,finished:'Game complete. The record is ready to verify.'}[room.phase];
  if(pending&&pending.code===room.code)$('table-actions').append(button('Retry previous move',()=>command(pending.action),'secondary'));
  if(room.phase==='lobby'&&room.host===me.id){const start=button('Start game',()=>command({type:'start'}));start.disabled=room.players.length<(room.options?.playerCount??2);$('table-actions').append(start);}
  if(room.phase==='committing'&&!room.deck.acknowledgements.includes(me.id))$('table-actions').append(button('Save & acknowledge deck',async()=>{saveReceipt();await command({type:'acknowledge',shuffleId:room.deck.shuffleId,commitment:room.deck.commitment});}));
  const hand=sortedHand(room.table?.hand??[],sortMode,room.game);$('hand-zone').hidden=hand.length===0;$('hand-count').textContent=`· ${hand.length} cards`;
  $('sort-hand').textContent=`↕ Sorted by ${sortMode}`;$('sort-hand').setAttribute('aria-label',`Sort hand by ${sortMode==='suit'?'rank':'suit'}`);
  $('hand').replaceChildren(...hand.map(code=>{const card=playingCard(code,{interactive:true,selected:selected===code});card.onclick=()=>run(()=>{selected=code;render(room);});return card;}));
  $('audit').disabled=false;$('audit').textContent='Download audit record';
  if(room.phase==='playing')for(const action of room.table?.actions??[]){const e=button(action.label,()=>command({type:action.type,...(action.input==='card'?{card:selected}:{})}));e.disabled=action.input==='card'&&!selected;$('game-actions').append(e);}
  $('commitment').textContent=room.deck?`Shuffle ${room.deck.shuffleId}\nSHA-256 ${room.deck.commitment}`:'The deck hash will appear before dealing.';
  if(room.table?.kind==='thousand'){
    renderThousand({room,me,selected,send:action=>command({type:'thousand',action}),refresh:()=>run(refreshRoom),run});
    if(room.table.phase==='trick')for(const card of $('hand').children){card.disabled=!room.table.controls.legalCards.includes(card.dataset.card);card.classList.toggle('is-unplayable',room.table.turn===me.id&&card.disabled);}
    $('commitment').textContent=room.deck?`Deal ${room.table.deal} · SHA-256 ${room.deck.commitment}`:'The dealer will publish the deck hash before dealing.';
  }else{
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
$('create-form').onsubmit=e=>{e.preventDefault();run(async()=>{const data=new FormData(e.target),game=data.get('game');render(await api('/rooms',{game,...(game==='thousand'?{options:{playerCount:Number(data.get('playerCount')),barrelThreshold:Number(data.get('barrelThreshold'))}}:{})}));});};
$('games').onchange=()=>{$('thousand-settings').hidden=$('games').value!=='thousand';};
$('join-form').onsubmit=e=>{e.preventDefault();run(async()=>render(await api(`/rooms/${String(new FormData(e.target).get('code')).trim().toUpperCase()}/join`,{})));};
$('back').onclick=lobby;
$('sort-hand').onclick=()=>{sortMode=sortMode==='suit'?'rank':'suit';localStorage.setItem('pastimer-hand-sort',sortMode);if(room)render(room);};
$('copy-code').onclick=()=>run(async()=>{if(navigator.clipboard){await navigator.clipboard.writeText(room.code);notice('Table code copied. Send it to your friends.');}else notice(`Your table code is ${room.code}`);});
window.addEventListener('resize',layoutHand);
$('audit').onclick=()=>run(async()=>{ const audit=await api(`/rooms/${room.code}/audit`); const receipts=JSON.parse(localStorage.getItem(`pastimer-receipts:${room.id}`)||'[]');const url=URL.createObjectURL(new Blob([JSON.stringify({...audit,receipts},null,2)],{type:'application/json'}));const a=element('a','');a.href=url;a.download=`pastimer-${room.code}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000); });
await run(async()=>{
  const games=await api('/games'); $('games').replaceChildren(...games.map(g=>{const e=element('option',g.name);e.value=g.id;return e;})); $('games').value='thousand'; $('games').onchange();
  try { signedIn(await api('/me')); } catch {
    const tg=window.Telegram?.WebApp;
    if(tg?.initData){tg.ready();tg.expand();tg.setHeaderColor?.('#101e28');tg.setBackgroundColor?.('#101e28');signedIn(await api('/auth',{initData:tg.initData}));}
    else {$('login').hidden=false;return;}
  }
  const previous=sessionStorage.getItem('room'); if(previous) render(await api(`/rooms/${previous}`));
});
async function refreshRoom(){if(!room)return;const code=room.code;const next=await api(`/rooms/${code}`);if(room?.code===code&&next.version!==room.version)render(next);}
setInterval(()=>{if(room&&!busy)run(refreshRoom);},1000);
