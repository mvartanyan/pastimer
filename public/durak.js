import {playingCard,cardBack} from '/cards.js';
const node=(tag,text='',cls='')=>{const e=document.createElement(tag);e.textContent=text;if(cls)e.className=cls;return e;};
const suits={C:'♣',D:'♦',H:'♥',S:'♠'};
const legalTurn=c=>c.attackCards.length||c.defences.length||c.canTake||c.canPass;
let transfer=false,multiple=false,chosen=[],target=null,tableKey='',pointer=null,swiped=false,deferred=null;
export function deferDurakRender(next,render){if(pointer){deferred=()=>render(next);return true;}return false;}
function release(){pointer=null;const task=deferred;deferred=null;setTimeout(()=>{task?.();swiped=false;},0);}
window.addEventListener('pointerup',release);window.addEventListener('pointercancel',()=>{swiped=true;release();});
window.addEventListener('pointermove',e=>{if(pointer&&Math.hypot(e.clientX-pointer.x,e.clientY-pointer.y)>8){swiped=true;if(pointer.type==='mouse')pointer.root.scrollLeft=pointer.left+pointer.x-e.clientX;}});
function revealPair(pairs,card){
  if(!card)return;const r=pairs.getBoundingClientRect(),b=card.getBoundingClientRect();
  if(b.left<r.left+4)pairs.scrollLeft-=r.left+4-b.left;
  else if(b.right>r.right-4)pairs.scrollLeft+=b.right-r.right+4;
}
export function layoutDurak(){const pairs=document.querySelector('#room[data-game^="durak-"] .durak-pairs');if(!pairs)return;const target=pairs.querySelector('.is-target');revealPair(pairs,target);const card=target?.querySelector('.durak-attack'),rail=pairs.closest('.table-rail');if(card&&rail){const r=rail.getBoundingClientRect(),b=card.getBoundingClientRect();if(b.bottom>r.bottom-18)rail.scrollTop+=b.bottom-r.bottom+18;else if(b.top<r.top+4)rail.scrollTop-=r.top+4-b.top;}}
export function durakSeats(room,me){
  const t=room.table;
  return room.players.map(p=>{const seat=t?.seats.find(s=>s.id===p.id),self=p.id===me.id,box=node('div','',`seat durak-seat${t?.turn===p.id?' is-turn':''}`);box.dataset.player=p.id;box.title=p.name;box.classList.toggle('is-defender',t?.defender===p.id);box.classList.toggle('is-self',self);
    const info=node('div','','seat-info');info.append(node('div',self?'You':p.name,'seat-name'),node('span',seat?.finished?'OUT':t?.defender===p.id?'DEFENDER':t?.attacker===p.id?'ATTACKER':'','seat-role'));
    box.append(info);if(seat){const count=node('span','','seat-cards');count.append(cardBack(),node('span',String(seat.count)));box.append(count);}return box;});
}
export function renderDurak({room,me,send,run,repaint,sortMode,sortedHand}){
  const t=room.table,c=t.controls,root=document.getElementById('hand'),surface=document.getElementById('thousand-table'),actions=document.getElementById('game-actions');
  const key=`${room.id}:${t.deal}:${t.attacker}:${t.defender}:${t.pairs.length}`;
  const changed=tableKey!==key;
  if(changed){tableKey=key;transfer=false;multiple=false;chosen=[];target=null;}
  if(!c.transferCards.length)transfer=false;
  const name=id=>room.players.find(p=>p.id===id)?.name??id;
  const button=(label,handler,cls='secondary')=>{const b=node('button',label,cls);b.type='button';b.onclick=()=>run(handler);return b;};
  const localButton=(label,handler)=>{const b=node('button',label,'secondary');b.type='button';b.onclick=handler;return b;};
  const dialog=document.getElementById('durak-dialog'),choices=document.getElementById('durak-choices');
  if(dialog.dataset.room!==room.id){dialog.close();dialog.dataset.room=room.id;}
  const menuButton=(label,handler,cls='secondary')=>{const b=localButton(label,()=>{dialog.close();handler();});b.className=cls;return b;};
  const openChoices=kind=>{dialog.dataset.kind=kind;fillChoices();if(!dialog.open)dialog.showModal();};
  const fillChoices=()=>{choices.replaceChildren();document.getElementById('durak-dialog-title').textContent=dialog.dataset.kind==='requests'?'Throw-in requests':dialog.dataset.kind==='request'?'Choose a rank':'Hand options';
    if(dialog.dataset.kind==='requests'){choices.append(node('p','Allow a request, or close this and keep playing.'));for(const request of t.requests)choices.append(button(`Allow ${name(request.player)}: ${request.rank}`,async()=>{await send({type:'approve',player:request.player,rank:request.rank});},''));if(!t.requests.length)choices.append(node('p','No pending requests.'));}
    else if(dialog.dataset.kind==='request'){for(const rank of c.requestRanks)choices.append(menuButton(`I want to throw in: ${rank}`,()=>run(()=>send({type:'request',rank})),''));}
    else {choices.append(menuButton(`Sort: ${sortMode}`,()=>document.getElementById('sort-hand').click()));if(!t.pairs.length&&c.attackCards.length)choices.append(menuButton('Multiple cards',()=>{multiple=!multiple;chosen=[];repaint();}));}
  };
  if(dialog.open){if(t.phase==='finished')dialog.close();else fillChoices();}
  const previousPairs=surface.querySelector('.durak-pairs'),previousPairScroll=previousPairs?.scrollLeft??0,previousTarget=changed?null:previousPairs?.querySelector('.is-target')?.dataset.attack,previousCount=previousPairs?.children.length??0;
  surface.hidden=false;surface.className='durak-table';surface.replaceChildren();
  document.getElementById('marriage-options').hidden=true;
  const phase=document.getElementById('phase');
  let heading='',hint='';
  const info=node('div','','durak-info'),trump=node('div','','durak-trump');
  if(t.trump){trump.append(playingCard(t.trump),node('span',`Trump: ${t.trump.slice(0,-1)}${suits[t.trump.slice(-1)]}`));}
  info.append(trump,node('span',`Stock: ${t.stockCount}`),node('span',`Attack: ${t.pairs.length}/${t.limit}`));surface.append(info);
  const pairs=node('div','','durak-pairs');
  const uncovered=c.defences.map(d=>d.attack);if(!uncovered.includes(target))target=c.defences.find(d=>d.cards.length)?.attack??uncovered[0]??null;
  for(const pair of t.pairs){const box=node('div','',`durak-pair${pair.card===target?' is-target':''}`),attack=node('button','','durak-attack');box.dataset.attack=pair.card;attack.type='button';attack.setAttribute('aria-label',`Defend against ${pair.card}`);attack.disabled=!uncovered.includes(pair.card);attack.append(playingCard(pair.card));attack.onclick=()=>{target=pair.card;transfer=false;repaint();};box.append(attack);if(pair.cover){const cover=playingCard(pair.cover);cover.classList.add('durak-cover');box.append(cover);}box.append(node('span',`${pair.card.slice(0,-1)}${suits[pair.card.slice(-1)]}${pair.cover?' / '+pair.cover.slice(0,-1)+suits[pair.cover.slice(-1)]:''}`,'durak-pair-label'));box.classList.toggle('is-beaten',!!pair.cover);pairs.append(box);}
  surface.append(pairs);pairs.scrollLeft=previousPairScroll;if(target&&target!==previousTarget)layoutDurak();else if(!target&&t.pairs.length>previousCount)revealPair(pairs,pairs.lastElementChild);
  if(!t.pairs.length && t.phase!=='finished')surface.append(node('p','Cards in this attack appear here.','durak-status'));
  if(t.phase==='finished'){
    surface.append(node('h2',t.loser?`${name(t.loser)} is the durak.`:'Draw.','durak-result'));
    if(room.host===me.id)actions.append(button('Rematch',()=>send({type:'rematch'}),''));
  }else{
    if(c.canTake)actions.append(button('Take',()=>send({type:'take'})));
    if(c.transferCards.length){const b=localButton('Transfer attack',()=>{transfer=!transfer;multiple=false;repaint();});b.setAttribute('aria-pressed',String(transfer));actions.append(b);}
    if(multiple){const play=button(`Attack (${chosen.length})`,()=>send({type:'attack',cards:[...chosen]}),'');play.disabled=!chosen.length;actions.prepend(play);actions.append(localButton('Cancel',()=>{multiple=false;chosen=[];repaint();}));}
    if(c.canPass)actions.append(button('Done throwing in',()=>send({type:'pass'})));
    if(c.requestRanks.length)actions.append(localButton('I want to throw in',()=>openChoices('request')));
    if(t.attacker===me.id&&t.requests.length)actions.append(localButton(`Requests: ${t.requests.length}`,()=>openChoices('requests')));
  }
  document.getElementById('durak-hand-options')?.remove();const options=localButton('Options',()=>openChoices('options'));options.id='durak-hand-options';document.querySelector('.hand-tools').append(options);
  if(t.phase==='finished'){heading=t.loser?`${name(t.loser)} is the durak.`:'Draw.';hint='The host can start a rematch.';}
  else if(transfer){heading='Transfer attack';hint='Tap a matching card to pass the attack on.';}
  else if(multiple){heading='Choose matching cards';hint='Tap cards of one rank, then Attack.';}
  else if(t.taking&&t.defender===me.id){heading='You are taking.';hint='Wait for the attackers to finish.';}
  else if(c.defences.length||c.canTake){heading='Your defence';hint=target?`Beat ${target.slice(0,-1)}${suits[target.slice(-1)]}. Tap a highlighted card or Take.`:'Take the cards to continue.';}
  else if(c.attackCards.length){heading=t.pairs.length?'Throw in or finish':'Your attack';hint=t.pairs.length?'Tap a highlighted card or Done throwing in.':'Tap any card to start. Options lets you play several.';}
  else if(c.canPass){heading='Finish your turn';hint='Press Done throwing in to continue.';}
  else if(t.requests.some(r=>r.player===me.id)){heading='Request sent';hint=`Waiting for ${name(t.attacker)} to allow your rank.`;}
  else if(t.seats.find(s=>s.id===me.id)?.finished){heading='You are out';hint='The other players are finishing.';}
  else {const waiting=t.pairs.some(p=>!p.cover)&&!t.taking?t.defender:t.turn??t.attacker;heading=t.taking?`${name(t.defender)} is taking.`:`Waiting for ${name(waiting)}`;hint=c.requestRanks.length?'Ask to throw in using the button below.':t.approvals.some(a=>a.player===me.id)?'Permission granted. Wait for your turn.':'Playable cards will light up on your turn.';}
  phase.replaceChildren(node('strong',heading),node('span',hint));
  document.getElementById('room').dataset.durakActive=String(!!legalTurn(c));
  const legal=transfer?c.transferCards:c.attackCards.length?c.attackCards:(c.defences.find(d=>d.attack===target)?.cards??[]);
  const left=root.dataset.table===room.id?root.scrollLeft:0;root.dataset.table=room.id;
  root.classList.add('durak-hand');root.onpointerdown=e=>{pointer={x:e.clientX,y:e.clientY,type:e.pointerType,root,left:root.scrollLeft};swiped=false;};
  const hand=sortedHand(t.hand,sortMode,room.game);
  if(sortMode==='suit'&&t.trump){const trumpSuit=t.trump.slice(-1);hand.sort((a,b)=>Number(a.endsWith(trumpSuit))-Number(b.endsWith(trumpSuit)));}
  root.replaceChildren(...hand.map(code=>{const card=playingCard(code,{interactive:true}),valid=legal.includes(code)&&(!multiple||!chosen.length||chosen[0].slice(0,-1)===code.slice(0,-1));card.disabled=!valid;
    card.classList.toggle('is-playable',valid);card.classList.toggle('is-unplayable',!valid);card.classList.toggle('selected',chosen.includes(code));
    card.onclick=()=>{if(swiped)return;if(multiple){if(chosen.includes(code))chosen=chosen.filter(c=>c!==code);else if(chosen.length<t.limit)chosen.push(code);repaint();return;}run(()=>send(transfer?{type:'transfer',card:code}:c.attackCards.includes(code)?{type:'attack',cards:[code]}:{type:'defend',attack:target,card:code}));};return card;}));
  root.scrollLeft=left;
}
