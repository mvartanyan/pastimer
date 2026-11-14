import {playingCard,cardBack,cardName} from '/cards.js';
const $=id=>document.getElementById(id);
const node=(tag,text='',className='')=>{const e=document.createElement(tag);e.textContent=text;if(className)e.className=className;return e;};
const symbols={H:'♥',D:'♦',C:'♣',S:'♠'};
let closureTimer,formKey='',draft={},openPiles=new Set(),previousPlays=new Set();
function label(text,input){const e=node('label',text);e.append(input);return e;}
export function clearThousand(){clearTimeout(closureTimer);$('thousand-table').hidden=true;$('trick-piles').hidden=true;}
export function renderThousand({room,me,selected,send,refresh,sendPlatform,run}){
  clearTimeout(closureTimer);const t=room.table,c=t.controls;
  const name=id=>room.players.find(p=>p.id===id)?.name??id;
  const btn=(text,action,className)=>{const e=node('button',text,className);e.type='button';e.onclick=()=>run(()=>send(action));return e;};
  const key=`${room.id}:${t.deal}:${t.phase}`;if(key!==formKey){formKey=key;draft={};}
  $('thousand-table').hidden=false;$('thousand-table').replaceChildren();$('trick-piles').hidden=false;
  const status={awaitShuffle:`${name(t.dealer)} deals next.`,auction:`${t.turn===me.id?'Your turn':name(t.turn)+'’s turn'} to bid.`,stock:`${name(t.contractor)} takes the stock.`,distribution:`${name(t.contractor)} is choosing cards to pass.`,preparation:`${name(t.contractor)} sets the final order.`,trick:t.turn===me.id?'Your turn. Choose a card.':`${name(t.turn)} is playing.`,closing:'Trick complete. Collecting the cards…',finished:'The match is complete.'};
  $('phase').textContent=status[t.phase];
  const hud=node('div','','table-hud');const trump=node('span',`TRUMP ${t.trump?symbols[t.trump]:'—'}`,'hud-pill');hud.append(trump);
  if(t.contractor)hud.append(node('span',`${name(t.contractor)} · ${t.order}${t.compulsory?' · compulsory':''}`,'hud-pill'));
  else hud.append(node('span',`FIRST TO 1,000`,'hud-pill'));
  $('thousand-table').append(hud);
  const center=node('div','','table-center');center.append(node('span','PASTIMER','table-watermark'));
  function stock(compact=false){
    const box=node('div','',compact?'stock-compact':'stock-display'),row=node('div','',`stock-row${t.stock?'':' sealed'}`);
    for(let i=0;i<3;i++)row.append(t.stock?playingCard(t.stock[i]):cardBack());
    box.append(row,node('div',t.stockTaken?'STOCK · TAKEN':t.stock?'STOCK · OPEN':'STOCK · SEALED','stock-caption'));return box;
  }
  if(t.phase==='auction'){
    const bid=node('div','','bid-emblem');bid.append(node('span','CURRENT BID','bid-caption'),node('strong',String(t.order),'bid-value'));center.append(bid,stock());
  }else if(['stock','distribution','preparation'].includes(t.phase)){
    center.append(stock(),node('p',t.phase==='stock'?'Three cards. A new possibility.':t.phase==='distribution'?'One card for each opponent.':`Your contract. Your opening lead.`,'table-instruction'));
  }else if(['trick','closing'].includes(t.phase)){
    if(t.stock)center.append(stock(true));
    const trick=node('div','','trick-area');trick.setAttribute('aria-label','Current trick');
    const currentKeys=new Set();
    for(const play of t.plays){const k=`${room.id}:${t.deal}:${play.player}:${play.card}`,slot=node('div','',`trick-play${previousPlays.has(k)?'':' new-play'}`);currentKeys.add(k);slot.append(node('small',name(play.player)),playingCard(play.card));trick.append(slot);}
    previousPlays=currentKeys;
    if(!t.plays.length){trick.append(node('p',t.turn===me.id?'Your lead.':'Waiting for the lead.','table-instruction'));}
    if(t.closeAt){const countdown=node('div','','trick-countdown');countdown.style.setProperty('--close-duration',`${Math.max(1,t.closeAt-t.serverNow)}ms`);countdown.append(node('span'));trick.append(countdown);closureTimer=setTimeout(refresh,Math.max(30,t.closeAt-t.serverNow+50));}
    center.append(trick);
  }else if(t.lastDeal){
    const result=node('div','','deal-summary');result.append(node('span',`DEAL ${t.lastDeal.deal} · ${t.lastDeal.status.toUpperCase()}`,'eyebrow'),node('h2',t.phase==='finished'?'A winning hand.':t.lastDeal.status==='cancelled'?'A fresh shuffle.':'One more hand?'));
    const scores=node('div','','result-scores');for(const player of room.players){const delta=t.lastDeal.after[player.id]-t.lastDeal.before[player.id],column=node('div','','result-score');column.append(node('span',player.name),node('strong',String(t.lastDeal.after[player.id])),node('small',`${delta>=0?'+':''}${delta} this deal`,delta<0?'negative':''));scores.append(column);}result.append(scores);center.append(result);
  }else{
    const deck=node('div','','stock-row sealed');for(let i=0;i<3;i++)deck.append(cardBack());center.append(deck,node('p','Good company. A fresh deck.','table-instruction'));
  }
  $('thousand-table').append(center);
  if(c.shuffle)$('table-actions').append(btn(t.deal?'Shuffle & deal next hand':'Shuffle & deal',{type:'shuffle'}));
  else if(t.phase==='awaitShuffle'&&room.computers?.includes(t.dealer)){const deal=node('button',t.deal?'Ask computer to deal next hand':'Ask computer to deal');deal.type='button';deal.onclick=()=>run(()=>sendPlatform({type:'computerDeal'}));$('table-actions').append(deal);}
  if(c.fourNines)$('table-actions').append(btn('Declare four nines',{type:'fourNines'},'secondary'));
  if(c.dismiss)$('table-actions').append(btn('Take a stick · палочка',{type:'dismiss'},'secondary'));
  if(c.pass)$('game-actions').append(btn('Pass',{type:'pass'},'secondary'));
  if(c.here)$('game-actions').append(btn(`Here · здесь · ${t.order}`,{type:'here'},'secondary'));
  function amountForm(text,min,max,value,action){
    const form=node('form','','inline-form'),input=node('input');input.type='number';input.min=String(min);input.max=String(max);input.step='5';input.required=true;input.value=String(draft.amount??value);input.oninput=()=>{draft.amount=input.value;};input.setAttribute('aria-label',text);
    const adjust=delta=>{input.value=String(Math.max(min,Math.min(max,(Number(input.value)||min)+delta)));draft.amount=input.value;};
    const minus=node('button','−','amount-step');minus.type='button';minus.setAttribute('aria-label','Decrease amount');minus.onclick=()=>adjust(-5);
    const plus=node('button','+','amount-step');plus.type='button';plus.setAttribute('aria-label','Increase amount');plus.onclick=()=>adjust(5);
    const submit=node('button',text);submit.type='submit';form.append(minus,input,plus,submit);form.onsubmit=e=>{e.preventDefault();run(()=>send({type:action,amount:Number(input.value)}));};$('game-actions').append(form);
  }
  if(c.bidMin!==null)amountForm('Raise bid',c.bidMin,c.bidMax,c.bidMin,'bid');
  if(c.takeStock)$('game-actions').append(btn('Take stock',{type:'takeStock'}));
  if(c.recipients.length){
    const form=node('form','','distribution-form');form.append(node('p','Choose a card for each friend. Keep it secret or show the table.'));
    const grid=node('div','','transfer-grid'),submit=node('button','Give cards');submit.type='submit';
    const inputs=c.recipients.map((recipient,i)=>{
      const select=node('select');select.required=true;for(const code of t.hand){const option=node('option',cardName(code));option.value=code;select.append(option);}select.value=draft[recipient.id]?.card??t.hand[i];
      const publicInput=node('input');publicInput.type='checkbox';publicInput.checked=draft[recipient.id]?.public??false;
      const preview=node('div','','transfer-preview');preview.append(playingCard(select.value));
      const update=()=>{draft[recipient.id]={card:select.value,public:publicInput.checked};preview.replaceChildren(playingCard(select.value));submit.disabled=inputs?.length===2&&inputs[0].select.value===inputs[1].select.value;};
      select.onchange=update;publicInput.onchange=update;
      const row=node('div','','transfer-row'),visibility=label('Give openly',publicInput);visibility.className='visibility-control';row.append(preview,label(`Card for ${recipient.name}`,select),visibility);grid.append(row);return {recipient,select,publicInput};
    });
    submit.disabled=inputs[0].select.value===inputs[1].select.value;form.append(grid,submit);form.onsubmit=e=>{e.preventDefault();run(()=>send({type:'distribute',transfers:inputs.map(({recipient,select,publicInput})=>({player:recipient.id,card:select.value,public:publicInput.checked}))}));};$('game-actions').append(form);
  }
  if(c.order)amountForm('Set order & start',t.order,c.bidMax,t.order,'order');
  if(c.legalCards.length){
    const play=btn('Play selected card',{type:'play',card:selected,announce:false});play.disabled=!selected||!c.legalCards.includes(selected);$('game-actions').append(play);
    if(selected&&c.announceCards.includes(selected))$('game-actions').append(btn(`Play & announce ${symbols[selected.slice(-1)]} marriage`,{type:'play',card:selected,announce:true},'secondary'));
  }
  if(t.transfers.length&&t.phase==='trick'){
    const info=node('div','','transfer-history');for(const transfer of t.transfers)info.append(node('span',`${cardName(transfer.card)} → ${name(transfer.player)} ${transfer.public?'◉':'· private'}`,'transfer-tag'));$('thousand-table').append(info);
  }
  $('trick-piles').replaceChildren();
  const heading=node('div','','pile-heading');heading.append(node('h3','Collected tricks'),node('span','Tap a pile to see its last trick'));$('trick-piles').append(heading);
  const piles=node('div','','pile-grid');piles.style.gridTemplateColumns=`repeat(${room.players.length===4?2:room.players.length},minmax(0,1fr))`;
  for(const pile of t.piles){
    const details=node('details','','pile');details.open=openPiles.has(pile.player);details.ontoggle=()=>{if(details.open)openPiles.add(pile.player);else openPiles.delete(pile.player);};
    const summary=node('summary'),stack=node('span','',`pile-stack${pile.count?'':' empty'}`);stack.append(cardBack(),cardBack());const title=node('span',name(pile.player)+(pile.player===me.id?' · you':''),'pile-title');title.append(node('strong',String(pile.count)));summary.append(stack,title);details.append(summary);
    const content=node('div','','pile-content');if(pile.top){content.append(node('p',`#${pile.top.number} · ${pile.top.points} pts · ${pile.top.trump?symbols[pile.top.trump]:'no trump'}`));const row=node('div','','last-trick-cards');for(const play of pile.top.plays){const slot=node('div','','pile-card');slot.append(playingCard(play.card),node('small',name(play.player)));row.append(slot);}content.append(row);}else content.append(node('p','No tricks yet.'));
    details.append(content);piles.append(details);
  }
  $('trick-piles').append(piles);
  $('audit').disabled=t.auditDeals===0;$('audit').textContent=t.auditDeals?`Download audit · ${t.auditDeals} settled deal${t.auditDeals===1?'':'s'}`:'Audit available after the deal';
}
