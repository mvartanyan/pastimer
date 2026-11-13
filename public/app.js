import {renderThousand,clearThousand} from '/thousand.js';
const $ = id => document.getElementById(id);
let me, room, selected, busy = false;
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
function lobby() { clearThousand(); document.querySelector('.intro').hidden=false; room = undefined; sessionStorage.removeItem('room'); $('room').hidden = true; $('lobby').hidden = false; }
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
function render(next) {
  if(room?.table?.deal!==next.table?.deal||!next.table?.hand?.includes(selected)) selected=undefined;
  room = next; $('room').dataset.phase=room.table?.phase??room.phase; $('room').dataset.version=String(room.version); $('room').dataset.game=room.game; document.querySelector('.intro').hidden=true; sessionStorage.setItem('room',room.code); $('lobby').hidden = true; $('room').hidden = false;
  $('room-code').textContent = room.code;
  $('seats').replaceChildren(...room.players.map(p => {const e=element('div',p.name+(p.id===me.id?' · you':''),'seat');const seat=room.table?.seats?.find(s=>s.id===p.id);e.append(element('small', seat ? `${seat.count} cards · ${seat.score} points${seat.sticks!==undefined?` · ${seat.sticks} sticks${seat.onBarrel?' · BARREL':''} · ${seat.role}`:''}${seat.played?' · ready':''}` : 'Waiting at the table'));return e;}));
  $('phase').textContent = {lobby:'Waiting for players. Share the table code.',committing:'A new deck is committed. Every player must acknowledge it.',playing:`Round ${room.table?.round ?? ''} · choose a card`,finished:'Game complete. Decks and moves are now revealed in the audit record.'}[room.phase];
  $('table-actions').replaceChildren();
  if(pending && pending.code===room.code) $('table-actions').append(button('Retry previous move',()=>command(pending.action),'secondary'));
  if(room.phase==='lobby' && room.host===me.id){const start=button('Start game',()=>command({type:'start'}));start.disabled=room.players.length<(room.options?.playerCount??2);$('table-actions').append(start);}
  if(room.phase==='committing' && !room.deck.acknowledgements.includes(me.id)) $('table-actions').append(button('Save & acknowledge deck',async()=>{
    saveReceipt(); await command({type:'acknowledge',shuffleId:room.deck.shuffleId,commitment:room.deck.commitment});
  }));
  const suits={C:'♣',D:'♦',H:'♥',S:'♠'};
  $('hand').replaceChildren(...(room.table?.hand??[]).map(card=>{ const suit=card.slice(-1);const e=button(card.slice(0,-1)+suits[suit],()=>{selected=card;render(room);},`card ${['D','H'].includes(suit)?'red':''} ${selected===card?'selected':''}`);e.setAttribute('aria-pressed',String(selected===card));e.dataset.card=card;return e;}));
  $('game-actions').replaceChildren();
  clearThousand(); $('audit').disabled=false; $('audit').textContent='Download audit record';
  if(room.phase==='playing') for(const a of room.table?.actions??[]) {const e=button(a.label,()=>command({type:a.type,...(a.input==='card'?{card:selected}:{})}));e.disabled=a.input==='card'&&!selected;$('game-actions').append(e);}
  $('commitment').textContent=room.deck?`Shuffle ${room.deck.shuffleId}\nSHA-256 ${room.deck.commitment}\nAcknowledged ${room.deck.acknowledgements.length}/${room.players.length}`:'Deck will be committed at game start.';
  if(room.table?.kind==='thousand') {
    renderThousand({room,me,selected,send:action=>command({type:'thousand',action}),refresh:()=>run(refreshRoom),run});
    if(room.table.phase==='trick') for(const card of $('hand').children) card.disabled=!room.table.controls.legalCards.includes(card.dataset.card);
    $('commitment').textContent=room.deck?`Deal ${room.table.deal} · SHA-256 ${room.deck.commitment}`:'The dealer will publish the deck hash before dealing.';
  }
  saveReceipt();
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
$('audit').onclick=()=>run(async()=>{ const audit=await api(`/rooms/${room.code}/audit`); const receipts=JSON.parse(localStorage.getItem(`pastimer-receipts:${room.id}`)||'[]');const url=URL.createObjectURL(new Blob([JSON.stringify({...audit,receipts},null,2)],{type:'application/json'}));const a=element('a','');a.href=url;a.download=`pastimer-${room.code}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000); });
await run(async()=>{
  const games=await api('/games'); $('games').replaceChildren(...games.map(g=>{const e=element('option',g.name);e.value=g.id;return e;})); $('games').value='thousand'; $('games').onchange();
  try { signedIn(await api('/me')); } catch {
    const tg=window.Telegram?.WebApp;
    if(tg?.initData){tg.ready();tg.expand();signedIn(await api('/auth',{initData:tg.initData}));}
    else {$('login').hidden=false;return;}
  }
  const previous=sessionStorage.getItem('room'); if(previous) render(await api(`/rooms/${previous}`));
});
async function refreshRoom(){if(!room)return;const code=room.code;const next=await api(`/rooms/${code}`);if(room?.code===code&&next.version!==room.version)render(next);}
setInterval(()=>{if(room&&!busy)run(refreshRoom);},1000);
