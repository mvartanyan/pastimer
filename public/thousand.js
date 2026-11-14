import {playingCard,cardBack,cardName} from '/cards.js';
const $=id=>document.getElementById(id);
const node=(tag,text='',className='')=>{const e=document.createElement(tag);e.textContent=text;if(className)e.className=className;return e;};
const symbols={H:'♥',D:'♦',C:'♣',S:'♠'};
let timer,formKey='',draft={},current,openPile=null;
const read=key=>{try{return JSON.parse(localStorage.getItem(key)||'{}');}catch{return {};}};
const deliveryKey=(room,me)=>`pastimer-deliveries:${me.id}:${room.id}:${room.table.deal}`;
function prepare(room){const key=`${room.id}:${room.table.deal}:${room.table.phase}`;if(key!==formKey){formKey=key;draft={transfers:{},target:0,announce:null};}}
export function incoming(room,me){
  if(!room.table?.deliveries)return [];
  const saved=read(deliveryKey(room,me));
  return room.table.deliveries.filter(d=>d.to===me.id&&d.card&&saved[d.id]!==2).map(d=>({...d,stage:saved[d.id]??(d.public?1:0)}));
}
export function selectedTransfers(room){prepare(room);return Object.values(draft.transfers??{}).map(t=>t.card).filter(Boolean);}
export function clearThousand(){clearTimeout(timer);$('thousand-table').hidden=true;}
export function inspectTrick(room,player){
  const pile=room.table?.piles?.find(p=>p.player===player);if(!pile?.top)return;
  const dialog=$('inspector'),content=$('inspector-content');content.replaceChildren();
  $('inspector-title').textContent=`Last trick won by ${room.players.find(p=>p.id===player)?.name??player}`;
  content.append(node('p',`Trick ${pile.top.number} - ${pile.top.points} points${pile.top.trump?` - ${symbols[pile.top.trump]} trump`:''}`));
  const cards=node('div','','inspect-cards');for(const play of pile.top.plays){const slot=node('div');slot.append(playingCard(play.card),node('span',room.players.find(p=>p.id===play.player)?.name??play.player));cards.append(slot);}content.append(cards);
  openPile={room:room.id,deal:room.table.deal};dialog.showModal();
}
export function handCard(code,{room,me,send,run,repaint}){
  prepare(room);const t=room.table,c=t.controls;
  if(c.recipients.length){
    const recipients=c.recipients;let target=recipients[draft.target??0]??recipients[0];
    const existing=Object.entries(draft.transfers).find(([,v])=>v.card===code);
    if(existing){delete draft.transfers[existing[0]];draft.target=recipients.findIndex(p=>p.id===existing[0]);}
    else{draft.transfers[target.id]={card:code,public:draft.transfers[target.id]?.public??false};const next=recipients.findIndex(p=>!draft.transfers[p.id]?.card);if(next>=0)draft.target=next;}
    repaint();return;
  }
  if(c.legalCards.includes(code))run(()=>send({type:'play',card:code,announce:c.announceCards.includes(code)&&draft.announce===code.slice(-1)}));
}
export function renderThousand({room,me,send,sendPlatform,refresh,run,repaint,now}){
  clearTimeout(timer);prepare(room);current={room,me};const t=room.table,c=t.controls;
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
  const status={awaitShuffle:`${name(t.dealer)} deals next.`,auction:ending?'Auction complete.':`${t.turn===me.id?'Your turn':name(t.turn)+"'s turn"} to bid.`,stock:`${name(t.contractor)} takes the stock.`,distribution:c.recipients.length?`Tap a card for ${name(c.recipients[draft.target??0]?.id??c.recipients[0].id)}.`:`${name(t.contractor)} is giving cards.`,preparation:`${name(t.contractor)} sets the contract.`,trick:t.turn===me.id?'Your turn - tap a card to play.':`${name(t.turn)} is playing.`,closing:'Trick complete.',finished:'The match is complete.'};
  $('phase').textContent=status[phase];
  $('thousand-table').hidden=false;$('thousand-table').replaceChildren();
  const hud=node('div','','table-hud');hud.append(node('span',`Trump ${t.trump?symbols[t.trump]:'none'}`,'hud-pill'));
  hud.append(node('span',t.contractor&&!ending?`${name(t.contractor)} - contract ${t.order}`:`Bid ${t.order}`,'hud-pill'));$('thousand-table').append(hud);
  const center=node('div','','table-center');
  function stock(){const box=node('div','','stock-display'),cards=node('div','','stock-row');cards.setAttribute('aria-label',t.stock?'Opened stock':'Sealed stock');for(let i=0;i<3;i++)cards.append(t.stock?playingCard(t.stock[i]):cardBack());box.append(cards,node('span',t.stock?'OPEN STOCK':'STOCK','stock-caption'));return box;}
  if(phase==='auction'){
    const bid=node('div','','bid-emblem');bid.append(node('span','CURRENT BID','bid-caption'),node('strong',String(t.order),'bid-value'));center.append(bid);
    if(!ending&&!t.stockTaken)center.append(stock());
  }else if(phase==='stock'){if(!t.stockTaken)center.append(stock());}
  else if(['distribution','preparation'].includes(phase)){center.append(node('p',phase==='distribution'?'Choose two cards from your hand.':`Contract ${t.order}`,'table-instruction'));}
  else if(['trick','closing'].includes(phase)){
    const trick=node('div','','trick-area');trick.setAttribute('aria-label','Current trick');
    for(const play of t.plays){const slot=node('div','','trick-play');slot.append(node('small',name(play.player)),playingCard(play.card));trick.append(slot);}
    if(!t.plays.length)trick.append(node('p',t.turn===me.id?'Your lead.':'Waiting for the lead.','table-instruction'));
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
    const form=node('form','','distribution-form'),grid=node('div','','transfer-grid');
    c.recipients.forEach((recipient,i)=>{
      const choice=draft.transfers[recipient.id],tile=node('div', '',`transfer-row${(draft.target??0)===i?' active-target':''}`),pick=node('button','','transfer-target');pick.type='button';pick.setAttribute('aria-label',`Choose a card for ${recipient.name}`);pick.onclick=()=>{draft.target=i;repaint();};
      pick.append(choice?.card?playingCard(choice.card):cardBack(),node('span',`To ${recipient.name}`));
      const open=node('input');open.type='checkbox';open.checked=choice?.public??false;open.setAttribute('aria-label',`Give openly to ${recipient.name}`);open.onchange=()=>{draft.transfers[recipient.id]={...draft.transfers[recipient.id],public:open.checked};};const label=node('label','Openly');label.append(open);tile.append(pick,label);grid.append(tile);
    });
    const submit=node('button','Give two cards');submit.disabled=!c.recipients.every(p=>draft.transfers[p.id]?.card);form.append(grid,submit);form.onsubmit=e=>{e.preventDefault();run(()=>send({type:'distribute',transfers:c.recipients.map(p=>({player:p.id,...draft.transfers[p.id]}))}));};$('game-actions').append(form);
  }
  const arrivals=incoming(room,me);$('incoming-zone').replaceChildren();
  for(const gift of arrivals){const button=node('button','','incoming-card');button.type='button';button.dataset.transfer=String(gift.id);button.setAttribute('aria-label',gift.stage?`Add ${cardName(gift.card)} to your hand`:`Reveal secret card from ${name(gift.from)}`);button.append(gift.stage?playingCard(gift.card):cardBack(),node('span',`${gift.public?'Open':'Secret'} from ${name(gift.from)}`),node('small',gift.stage?'Tap to add to hand':'Tap to reveal'));button.onclick=()=>{const saved=read(deliveryKey(room,me));saved[gift.id]=gift.stage?2:1;localStorage.setItem(deliveryKey(room,me),JSON.stringify(saved));repaint();};$('incoming-zone').append(button);}
  if(arrivals.length)$('phase').textContent='You received a card - tap it to receive it.';
  if(c.announceCards.length&&!arrivals.length){const suits=[...new Set(c.announceCards.map(card=>card.slice(-1)))];for(const suit of suits){const toggle=node('button',`Announce ${symbols[suit]} +${{H:100,D:80,C:60,S:40}[suit]}`,'secondary marriage-toggle');toggle.setAttribute('aria-pressed',String(draft.announce===suit));toggle.onclick=()=>{draft.announce=draft.announce===suit?null:suit;repaint();};$('game-actions').append(toggle);}}
  $('audit').disabled=t.auditDeals===0;$('audit').textContent=t.auditDeals?'Download audit':'No completed deal yet';
}
