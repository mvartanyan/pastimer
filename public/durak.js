import {playingCard,cardBack} from '/cards.js';
const node=(tag,text='',cls='')=>{const e=document.createElement(tag);e.textContent=text;if(cls)e.className=cls;return e;};
const suits={C:'♣',D:'♦',H:'♥',S:'♠'};
let transfer=false,multiple=false,chosen=[],target=null,tableKey='',pointer=null,swiped=false,deferred=null;
export function deferDurakRender(next,render){if(pointer){deferred=()=>render(next);return true;}return false;}
function release(){pointer=null;const task=deferred;deferred=null;setTimeout(()=>{task?.();swiped=false;},0);}
window.addEventListener('pointerup',release);window.addEventListener('pointercancel',()=>{swiped=true;release();});
window.addEventListener('pointermove',e=>{if(pointer&&Math.hypot(e.clientX-pointer.x,e.clientY-pointer.y)>8){swiped=true;if(pointer.type==='mouse')pointer.root.scrollLeft=pointer.left+pointer.x-e.clientX;}});
export function durakSeats(room,me){
  const t=room.table;
  return room.players.map(p=>{const seat=t?.seats.find(s=>s.id===p.id),self=p.id===me.id,box=node('div','',`seat durak-seat${t?.turn===p.id?' is-turn':''}`);box.dataset.player=p.id;
    const info=node('div','','seat-info');info.append(node('div',p.name+(self?' - you':''),'seat-name'),node('span',seat?.finished?'OUT':t?.defender===p.id?'DEFENDER':t?.attacker===p.id?'ATTACKER':'','seat-role'));
    box.append(info);if(seat){const count=node('span','','seat-cards');count.append(cardBack(),node('span',String(seat.count)));box.append(count);}return box;});
}
export function renderDurak({room,me,send,run,repaint,sortMode,sortedHand}){
  const t=room.table,c=t.controls,root=document.getElementById('hand'),surface=document.getElementById('thousand-table'),actions=document.getElementById('game-actions');
  const key=`${room.id}:${t.deal}:${t.attacker}:${t.defender}:${t.pairs.length}`;
  if(tableKey!==key){tableKey=key;transfer=false;multiple=false;chosen=[];target=null;}
  if(!c.transferCards.length)transfer=false;
  const name=id=>room.players.find(p=>p.id===id)?.name??id;
  const button=(label,handler,cls='secondary')=>{const b=node('button',label,cls);b.type='button';b.onclick=()=>run(handler);return b;};
  const localButton=(label,handler)=>{const b=node('button',label,'secondary');b.type='button';b.onclick=handler;return b;};
  surface.hidden=false;surface.className='durak-table';surface.replaceChildren();
  document.getElementById('marriage-options').hidden=true;
  document.getElementById('phase').textContent=t.phase==='finished'?(t.loser?`${name(t.loser)} is the durak.`:'Draw.') :t.taking?`${name(t.defender)} is taking.`:`${name(t.attacker)} attacks ${name(t.defender)}.`;
  const info=node('div','','durak-info'),trump=node('div','','durak-trump');
  if(t.trump){trump.append(playingCard(t.trump),node('span',`Trump: ${suits[t.trump.slice(-1)]}`));}
  info.append(trump,node('span',`Stock: ${t.stockCount}`),node('span',`Discard: ${t.discardCount}`));if(t.attacker===me.id&&t.requests.length)info.append(node('strong',`Requests: ${t.requests.length}`,'durak-request-count'));surface.append(info);
  const pairs=node('div','','durak-pairs');
  const uncovered=c.defences.map(d=>d.attack);if(!uncovered.includes(target))target=uncovered[0]??null;
  for(const pair of t.pairs){const box=node('div','',`durak-pair${pair.card===target?' is-target':''}`),attack=node('button','','durak-attack');attack.type='button';attack.setAttribute('aria-label',`Defend against ${pair.card}`);attack.disabled=!uncovered.includes(pair.card);attack.append(playingCard(pair.card));attack.onclick=()=>{target=pair.card;transfer=false;repaint();};box.append(attack);if(pair.cover){const cover=playingCard(pair.cover);cover.classList.add('durak-cover');box.append(cover);}pairs.append(box);}
  surface.append(pairs);
  if(!t.pairs.length && t.phase!=='finished')surface.append(node('p',`${name(t.attacker)} opens the attack.`,'durak-status'));
  if(t.phase==='finished'){
    surface.append(node('h2',t.loser?`${name(t.loser)} is the durak.`:'Draw.','durak-result'));
    if(room.host===me.id)actions.append(button('Rematch',()=>send({type:'rematch'}),''));
  }else{
    if(c.canTake)actions.append(button('Take',()=>send({type:'take'})));
    if(c.transferCards.length){const b=localButton('Transfer attack',()=>{transfer=!transfer;multiple=false;repaint();});b.setAttribute('aria-pressed',String(transfer));actions.append(b);}
    if(!t.pairs.length && c.attackCards.length){const b=localButton('Multiple cards',()=>{multiple=!multiple;chosen=[];repaint();});b.setAttribute('aria-pressed',String(multiple));actions.append(b);if(multiple){const play=button(`Attack (${chosen.length})`,()=>send({type:'attack',cards:[...chosen]}),'');play.disabled=!chosen.length;actions.append(play);}}
    if(c.canPass)actions.append(button('Done throwing in',()=>send({type:'pass'})));
    for(const rank of c.requestRanks)actions.append(button(`I want to throw in: ${rank}`,()=>send({type:'request',rank})));
    if(t.attacker===me.id)for(const request of t.requests)actions.append(button(`Allow ${name(request.player)}: ${request.rank}`,()=>send({type:'approve',player:request.player,rank:request.rank}),''));
    else for(const request of t.requests)actions.append(node('span',`Request pending: ${request.rank}`,'durak-pending'));
  }
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
