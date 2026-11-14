import {playingCard,cardBack,cardName} from '/cards.js';
const $=id=>document.getElementById(id);
const node=(tag,text='',className='')=>{const e=document.createElement(tag);e.textContent=text;if(className)e.className=className;return e;};
const symbols={H:'♥',D:'♦',C:'♣',S:'♠'};
let timer,formKey='',draft={},current,openPile=null,drag=null,suppressClickUntil=0,ringsId=0,collectionAnimations=[];
const read=key=>{try{return JSON.parse(localStorage.getItem(key)||'{}');}catch{return {};}};
const deliveryKey=(room,me)=>`pastimer-deliveries:${me.id}:${room.id}:${room.table.deal}`;
function prepare(room){const key=`${room.id}:${room.table.deal}:${room.table.phase}`;if(key!==formKey){formKey=key;draft={transfers:{},target:0,announce:{},announceVersion:room.version};}if(draft.announceVersion!==room.version){draft.announce={};draft.announceVersion=room.version;}}
export function incoming(room,me){
  // Receipt acknowledgment precedes the recipient's first play. Reopening a
  // later trick must not resurrect historical transfers or already played cards.
  if(!room.table?.deliveries||room.table.hand.length!==8||['awaitShuffle','finished'].includes(room.table.phase))return [];
  const saved=read(deliveryKey(room,me));
  return room.table.deliveries.filter(d=>d.to===me.id&&d.card&&room.table.hand.includes(d.card)&&saved[d.id]!==2).map(d=>({...d,stage:saved[d.id]??(d.public?1:0)}));
}
export function selectedTransfers(room){prepare(room);return Object.values(draft.transfers??{}).map(t=>t.card).filter(Boolean);}
export function clearThousand(){for(const animation of collectionAnimations)animation.cancel();collectionAnimations=[];cancelDrag();clearTimeout(timer);$('thousand-table').hidden=true;}
export function inspectTrick(room,player){
  const pile=room.table?.piles?.find(p=>p.player===player);if(!pile?.top)return;
  const dialog=$('inspector'),content=$('inspector-content');content.replaceChildren();
  $('inspector-title').textContent=`Last trick won by ${room.players.find(p=>p.id===player)?.name??player}`;
  content.append(node('p',`Trick ${pile.top.number} - ${pile.top.points} points${pile.top.trump?` - ${symbols[pile.top.trump]} trump`:''}`));
  const cards=node('div','','inspect-cards');for(const play of pile.top.plays){const slot=node('div');slot.append(playingCard(play.card),node('span',room.players.find(p=>p.id===play.player)?.name??play.player));cards.append(slot);}content.append(cards);
  openPile={room:room.id,deal:room.table.deal};dialog.showModal();
}
function placeCard(room,code,recipient){
  prepare(room);
  const previous=Object.entries(draft.transfers).find(([,choice])=>choice.card===code);
  const openly=previous?.[1].public??false;
  if(previous)delete draft.transfers[previous[0]];
  if(recipient)draft.transfers[recipient]={card:code,public:openly};
  const next=room.table.controls.recipients.findIndex(p=>!draft.transfers[p.id]);
  draft.target=next<0?0:next;
}
function cancelDrag(){
  if(!drag)return;
  drag.ghost?.remove();drag.source.classList.remove('is-dragging');
  document.querySelectorAll('.drop-hover').forEach(e=>e.classList.remove('drop-hover'));
  drag=null;
}
export function bindDistributionDrag({room,repaint}){
  const recipients=room.table?.controls.recipients??[];if(!recipients.length)return;
  for(const source of document.querySelectorAll('#hand .card,.seat-transfer')){
    source.ondragstart=e=>e.preventDefault();
    source.onpointerdown=e=>{
      if(e.button!==0||e.isPrimary===false)return;
      e.stopPropagation();cancelDrag();
      drag={source,code:source.dataset.transferCard??source.dataset.card,x:e.clientX,y:e.clientY,id:e.pointerId};
      source.setPointerCapture(e.pointerId);
    };
    source.onpointermove=e=>{
      if(!drag||drag.source!==source||drag.id!==e.pointerId)return;
      if(!drag.ghost&&Math.hypot(e.clientX-drag.x,e.clientY-drag.y)<8)return;
      e.preventDefault();
      if(!drag.ghost){drag.ghost=node('div','','transfer-drag');drag.ghost.append(source.matches('.seat-transfer')?source.firstElementChild.cloneNode(true):playingCard(drag.code));document.body.append(drag.ghost);source.classList.add('is-dragging');}
      drag.ghost.style.left=`${e.clientX}px`;drag.ghost.style.top=`${e.clientY}px`;
      document.querySelectorAll('.drop-hover').forEach(el=>el.classList.remove('drop-hover'));
      const target=document.elementFromPoint(e.clientX,e.clientY)?.closest('[data-recipient]');
      if(target)target.classList.add('drop-hover');
      else if(document.elementFromPoint(e.clientX,e.clientY)?.closest('#hand'))$('hand').classList.add('drop-hover');
    };
    source.onpointerup=e=>{
      if(!drag||drag.source!==source||drag.id!==e.pointerId)return;
      const moved=!!drag.ghost,code=drag.code,target=document.elementFromPoint(e.clientX,e.clientY),recipient=target?.closest('[data-recipient]')?.dataset.recipient;
      cancelDrag();
      if(!moved)return;
      suppressClickUntil=performance.now()+350;setTimeout(()=>suppressClickUntil=0,0);e.preventDefault();
      if(current.room.id!==room.id||current.room.table.phase!=='distribution')return;
      if(recipients.some(p=>p.id===recipient))placeCard(room,code,recipient);
      else if(target?.closest('#hand'))placeCard(room,code,null);
      repaint();
    };
    source.onpointercancel=source.onlostpointercapture=()=>cancelDrag();
  }
}
export function handCard(code,{room,me,send,run,repaint}){
  if(performance.now()<suppressClickUntil)return;
  prepare(room);const t=room.table,c=t.controls;
  if(c.recipients.length){
    placeCard(room,code,(c.recipients[draft.target??0]??c.recipients[0]).id);
    repaint();return;
  }
  if(c.legalCards.includes(code))run(()=>send({type:'play',card:code,announce:c.announceCards.includes(code)&&draft.announce[code.slice(-1)]!==false}));
}
function marriageRings(extra=''){
  const rings=node('span','',`marriage-rings ${extra}`.trim()),id=`ring-metal-${++ringsId}`;rings.setAttribute('aria-hidden','true');
  rings.innerHTML=`<svg viewBox="0 0 68 48" fill="none"><defs><linearGradient id="${id}" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#fff0bc"/><stop offset=".4" stop-color="#e9bb57"/><stop offset=".72" stop-color="#b37a20"/><stop offset="1" stop-color="#ffe6a0"/></linearGradient></defs><circle cx="24" cy="24" r="18" stroke="url(#${id})" stroke-width="4.5"/><path d="M62 24a18 18 0 1 1-36 0a18 18 0 1 1 36 0" stroke="#17332f" stroke-width="8"/><circle cx="44" cy="24" r="18" stroke="url(#${id})" stroke-width="4.5"/></svg>`;
  return rings;
}
export function layoutTrick(){
  if(!current||$('room').hidden)return;
  const {room,me,now}=current,t=room.table,surface=$('table-surface').getBoundingClientRect(),marker=$('thousand-table').querySelector('.trump-marker'),area=$('thousand-table').querySelector('.trick-area');
  const seats=[...$('seats').children].filter(e=>!e.classList.contains('seat-dealer')),own=seats.find(e=>e.classList.contains('seat-self')),others=seats.filter(e=>e!==own);
  const top=Math.max(0,...others.map(e=>e.getBoundingClientRect().bottom-surface.top))+8,bottom=(own?own.getBoundingClientRect().top-surface.top:surface.height)-8;
  const mr=marker&&!marker.hidden?marker.getBoundingClientRect():null,left=seats.find(e=>e.classList.contains('seat-left'))?.getBoundingClientRect(),right=seats.find(e=>e.classList.contains('seat-right'))?.getBoundingClientRect();
  const centredAbove=mr&&left&&right&&surface.width/2-mr.width/2>=left.right-surface.left+4&&surface.width/2+mr.width/2<=right.left-surface.left-4;
  const markerHeight=mr&&!centredAbove?mr.height:0;
  if(marker){marker.style.left=`${surface.width/2}px`;marker.style.top=`${centredAbove?Math.min(left.top,right.top)-surface.top+3:top}px`;}
  if(!area)return;
  const available=bottom-top,width=Math.max(38,Math.min(84,surface.width*.2,(available-markerHeight-20)*5/7)),height=width*7/5;
  for(const animation of collectionAnimations)animation.cancel();collectionAnimations=[];
  for(const slot of area.querySelectorAll('.trick-play')){
    const seat=seats.find(e=>e.dataset.player===slot.dataset.player),self=seat===own,left=seat?.classList.contains('seat-left');
    const x=surface.width*(self ? 0.5 : left ? 0.18 : 0.82),y=self?bottom-height/2-5:top+height/2+5,angle=self?0:left?-4:4;
    slot.style.left=`${x}px`;slot.style.top=`${y}px`;slot.style.setProperty('--played-card-width',`${width}px`);slot.style.setProperty('--owner-angle',`${angle}deg`);
    slot.dataset.position=self?'self':left?'left':'right';
    if(t.closeAt&&t.trickWinner&&!matchMedia('(prefers-reduced-motion: reduce)').matches){
      const target=seats.find(e=>e.dataset.player===t.trickWinner)?.querySelector('.seat-hand')?.getBoundingClientRect(),remaining=t.closeAt-now();
      if(target&&remaining>90){const dx=target.left+target.width/2-surface.left-x,dy=target.top+target.height/2-surface.top-y,delay=Math.max(0,remaining-800),duration=Math.min(720,remaining-80);slot.dataset.collectingTo=t.trickWinner;
        collectionAnimations.push(slot.animate([{transform:`translate(-50%,-50%) rotate(${angle}deg)`,opacity:1},{transform:`translate(calc(-50% + ${dx}px),calc(-50% + ${dy}px)) scale(.16) rotate(0deg)`,opacity:.25}],{delay,duration,easing:'cubic-bezier(.4,0,.2,1)',fill:'forwards'}));}
    }
  }
}
export function renderThousand({room,me,send,sendPlatform,refresh,run,repaint,now}){
  clearTimeout(timer);prepare(room);current={room,me,now};const t=room.table,c=t.controls;
  if(openPile&&(openPile.room!==room.id||openPile.deal!==t.deal)){if($('inspector').open)$('inspector').close();openPile=null;}
  const name=id=>room.players.find(p=>p.id===id)?.name??id;
  const btn=(text,action,secondary=false)=>{const e=node('button',text,secondary?'secondary':'');e.type='button';e.onclick=()=>run(()=>send(action));return e;};
  const calls=room.auctionCalls??[],last=calls.at(-1),callUntil=last?last.at+3000:0;
  const ending=t.phase==='stock'&&callUntil>now(),auction=t.phase==='auction'||ending;
  const schedule=at=>{const delay=at-now();if(delay>0)timer=setTimeout(repaint,delay+25);};
  if(auction){
    const latest=new Map(calls.map(call=>[call.player,call]));
    for(const [player,call]of latest){const seat=[...$('seats').children].find(e=>e.dataset.player===player);if(!seat)continue;
      const text=call.kind==='auction.pass'?'Pass':call.kind==='auction.here'?`Here - ${call.amount}`:String(call.amount);
      const bubble=node('span',text,`call-bubble${call===last&&callUntil>now()?' speaking':''}`);bubble.dataset.seq=String(call.seq);bubble.setAttribute('aria-label',`${name(player)}: ${text}`);seat.append(bubble);
    }
    if(callUntil>now())schedule(callUntil);
  }
  const phase=ending?'auction':t.phase;
  $('room').dataset.presentation=phase;
  const status={awaitShuffle:`${name(t.dealer)} deals next.`,auction:ending?'Auction complete.':`${t.turn===me.id?'Your turn':name(t.turn)+"'s turn"} to bid.`,stock:`${name(t.contractor)} takes the stock.`,distribution:c.recipients.length?'Give one card to each hand.':`${name(t.contractor)} is giving cards.`,preparation:`${name(t.contractor)} sets the contract.`,trick:t.turn===me.id?'Your turn - tap a card to play.':`${name(t.turn)} is playing.`,closing:`${name(t.trickWinner??'')} takes the trick.`,finished:'The match is complete.'};
  $('phase').textContent=status[phase];
  $('thousand-table').hidden=false;$('thousand-table').replaceChildren();
  const hud=node('div','','table-hud'),trump=node('div','','trump-marker');trump.dataset.trump=t.trump??'none';trump.setAttribute('aria-label',t.trump?`Current trump: ${{H:'hearts',D:'diamonds',C:'clubs',S:'spades'}[t.trump]}`:'No trump');
  if(t.trump){const mark=node('span',symbols[t.trump],`trump-suit suit-${t.trump}`);mark.setAttribute('aria-hidden','true');trump.append(marriageRings(),mark);}else trump.append(node('span','NO TRUMP'));
  trump.hidden=!['trick','closing'].includes(phase)&&!t.trump;hud.append(trump);$('thousand-table').append(hud);
  const center=node('div','','table-center');
  function stock(){const box=node('div','','stock-display'),cards=node('div','','stock-row');cards.setAttribute('aria-label',t.stock?'Opened stock':'Sealed stock');for(let i=0;i<3;i++)cards.append(t.stock?playingCard(t.stock[i]):cardBack());box.append(cards,node('span',t.stock?'OPEN STOCK':'STOCK','stock-caption'));return box;}
  if(phase==='auction'){
    const bid=node('div','','bid-emblem');bid.append(node('span','CURRENT BID','bid-caption'),node('strong',String(t.order),'bid-value'));center.append(bid);
    if(!ending&&!t.stockTaken)center.append(stock());
  }else if(phase==='stock'){if(!t.stockTaken)center.append(stock());}
  else if(['distribution','preparation'].includes(phase)){}
  else if(['trick','closing'].includes(phase)){
    const trick=node('div','','trick-area');trick.setAttribute('aria-label','Current trick');
    for(const play of t.plays){const slot=node('div','',`trick-play${play.player===t.trickLeader?' is-lead':''}${play.player===t.trickWinner?' is-winning':''}`);slot.dataset.player=play.player;slot.setAttribute('aria-label',`${name(play.player)} played ${cardName(play.card)}${play.player===t.trickLeader?' - first card':''}`);slot.append(playingCard(play.card));if(play.card===t.announcingCard){slot.classList.add('declares-marriage');slot.append(marriageRings('card-marriage'));slot.setAttribute('aria-description','Marriage announced');}trick.append(slot);}
    if(t.closeAt){const countdown=node('div','','trick-countdown');countdown.style.setProperty('--close-duration',`${Math.max(1,t.closeAt-now())}ms`);countdown.append(node('span'));trick.append(countdown);timer=setTimeout(refresh,Math.max(30,t.closeAt-now()+40));}
    center.append(trick);
  }else if(t.lastDeal){
    const summary=node('div','','deal-summary');summary.append(node('span',`DEAL ${t.lastDeal.deal} - ${t.lastDeal.status.toUpperCase()}`,'eyebrow'),node('h2',phase==='finished'?'A winning hand.':'One more hand?'));
    const scores=node('div','','result-scores');for(const p of room.players){const d=t.lastDeal.after[p.id]-t.lastDeal.before[p.id],column=node('div','','result-score');column.append(node('span',p.name),node('strong',String(t.lastDeal.after[p.id])),node('small',`${d>=0?'+':''}${d}`));scores.append(column);}summary.append(scores);center.append(summary);
  }else center.append(node('p','A fresh deck.','table-instruction'));
  $('thousand-table').append(center);
  if(c.shuffle)$('table-actions').append(btn(t.deal?'Shuffle & deal next hand':'Shuffle & deal',{type:'shuffle'}));
  else if(t.phase==='awaitShuffle'&&room.computers?.includes(t.dealer)){const deal=node('button',t.deal?'Ask computer to deal next hand':'Ask computer to deal');deal.type='button';deal.onclick=()=>run(()=>sendPlatform({type:'computerDeal'}));$('table-actions').append(deal);}
  if(c.fourNines)$('game-actions').append(btn('Four nines',{type:'fourNines'},true));
  if(c.dismiss&&!ending)$('game-actions').append(btn('Take a stick',{type:'dismiss'},true));
  function amountForm(text,min,max,value,action){
    const form=node('form','','inline-form'),input=node('input');input.type='number';input.min=String(min);input.max=String(max);input.step='5';input.required=true;input.value=String(draft.amount??value);input.oninput=()=>{draft.amount=input.value;};input.setAttribute('aria-label',text);
    const adjust=delta=>{input.value=String(Math.max(min,Math.min(max,(Number(input.value)||min)+delta)));draft.amount=input.value;};
    const minus=node('button','-','amount-step');minus.type='button';minus.setAttribute('aria-label','Decrease amount');minus.onclick=()=>adjust(-5);
    const plus=node('button','+','amount-step');plus.type='button';plus.setAttribute('aria-label','Increase amount');plus.onclick=()=>adjust(5);
    const submit=node('button',text==='Raise bid'?'Bid':'Start');submit.setAttribute('aria-label',text);submit.type='submit';form.append(minus,input,plus,submit);form.onsubmit=e=>{e.preventDefault();run(()=>send({type:action,amount:Number(input.value)}));};$('game-actions').append(form);return form;
  }
  if(!ending){
    if(c.pass)$('game-actions').append(btn('Pass',{type:'pass'},true));
    if(c.here)$('game-actions').append(btn(`Here - ${t.order}`,{type:'here'},true));
    if(c.bidMin!==null)amountForm('Raise bid',c.bidMin,c.bidMax,c.bidMin,'bid');
    if(c.takeStock)$('game-actions').append(btn('Take stock',{type:'takeStock'}));
    if(c.order)amountForm('Set order & start',t.order,c.bidMax,t.order,'order');
  }
  if(t.phase==='auction'&&callUntil>now())for(const element of $('game-actions').querySelectorAll('button,input'))element.disabled=true;
  if(c.recipients.length){
    for(const [index,recipient]of c.recipients.entries()){
      const seat=[...$('seats').children].find(e=>e.dataset.player===recipient.id),choice=draft.transfers[recipient.id];
      if(!seat)continue;
      seat.dataset.recipient=recipient.id;seat.classList.add('transfer-recipient');
      const hand=seat.querySelector('.seat-hand');hand.disabled=false;hand.setAttribute('aria-label',`Give a card to ${recipient.name}`);hand.onclick=e=>{e.stopPropagation();draft.target=index;repaint();};
      if(choice){
        const card=node('button','','seat-transfer');card.type='button';card.dataset.transferCard=choice.card;card.dataset.visibility=choice.public?'open':'secret';card.setAttribute('aria-label',`Card for ${recipient.name} - ${choice.public?'open, turn face down':'secret, turn face up'}`);card.setAttribute('aria-pressed',String(choice.public));card.append(choice.public?playingCard(choice.card):cardBack());
        card.onclick=e=>{e.stopPropagation();if(performance.now()<suppressClickUntil)return;choice.public=!choice.public;repaint();};seat.append(card);
      }
    }
    const give=btn('Give two cards',{type:'distribute',transfers:c.recipients.map(p=>({player:p.id,...draft.transfers[p.id]}))});give.disabled=!c.recipients.every(p=>draft.transfers[p.id]?.card);$('table-actions').append(give);
  }
  const arrivals=incoming(room,me);$('incoming-zone').replaceChildren();
  for(const gift of arrivals){const button=node('button','','incoming-card');button.type='button';button.dataset.transfer=String(gift.id);button.setAttribute('aria-label',gift.stage?`Add ${cardName(gift.card)} to your hand`:'Reveal secret card');button.append(gift.stage?playingCard(gift.card):cardBack());button.onclick=()=>{const saved=read(deliveryKey(room,me));saved[gift.id]=gift.stage?2:1;localStorage.setItem(deliveryKey(room,me),JSON.stringify(saved));repaint();};$('incoming-zone').append(button);}
  if(arrivals.length){$('phase').textContent='Your card.';for(const control of $('game-actions').querySelectorAll('button,input'))control.disabled=true;}
  if(c.announceCards.length&&!arrivals.length){const suits=[...new Set(c.announceCards.map(card=>card.slice(-1)))];for(const suit of suits){const toggle=node('button',`${symbols[suit]} +${{H:100,D:80,C:60,S:40}[suit]}`,'secondary marriage-toggle');toggle.setAttribute('aria-label',`Announce ${symbols[suit]} +${{H:100,D:80,C:60,S:40}[suit]}`);toggle.setAttribute('aria-pressed',String(draft.announce[suit]!==false));toggle.onclick=()=>{draft.announce[suit]=draft.announce[suit]===false;repaint();};$('game-actions').append(toggle);}}
  for(const card of $('hand').children){const announce=c.announceCards.includes(card.dataset.card)&&draft.announce[card.dataset.card.slice(-1)]!==false;card.classList.toggle('announces-marriage',announce);if(announce)card.setAttribute('aria-description','Announces the marriage and changes trump');}
  $('audit').disabled=t.auditDeals===0;$('audit').textContent=t.auditDeals?'Download audit':'No completed deal yet';
}
