const $ = id => document.getElementById(id);
const node = (tag,text='',className='') => {const e=document.createElement(tag);e.textContent=text;if(className)e.className=className;return e;};
const symbols={H:'♥',D:'♦',C:'♣',S:'♠'};
export const cardName=card=>card.slice(0,-1)+symbols[card.slice(-1)];
let closureTimer,formKey='',draft={},openPiles=new Set();
function label(text,input){const e=node('label',text);e.append(input);return e;}
function cardsRow(cards){const row=node('div','','mini-cards');for(const card of cards){row.append(node('span',cardName(card),`mini-card ${['H','D'].includes(card.slice(-1))?'red':''}`));}return row;}
export function clearThousand(){clearTimeout(closureTimer);$('thousand-table').hidden=true;$('trick-piles').hidden=true;}
export function renderThousand({room,me,selected,send,refresh,run}){
  clearTimeout(closureTimer);
  const t=room.table,c=t.controls;
  const name=id=>room.players.find(p=>p.id===id)?.name??id;
  const btn=(text,action,className)=>{const e=node('button',text,className);e.onclick=()=>run(()=>send(action));return e;};
  const key=`${room.id}:${t.deal}:${t.phase}`;if(key!==formKey){formKey=key;draft={};}
  $('thousand-table').hidden=false;$('thousand-table').replaceChildren();$('trick-piles').hidden=false;
  const status={awaitShuffle:`${name(t.dealer)} is dealing next.`,auction:`${name(t.turn)} to bid. Current bid: ${t.order}.`,stock:`${name(t.contractor)} takes the stock.`,distribution:`${name(t.contractor)} distributes one card to each opponent.`,preparation:`${name(t.contractor)} sets the final order.`,trick:`${name(t.turn)} to play.`,closing:'Trick complete · collecting after 3 seconds.',finished:'Match complete.'};
  $('phase').textContent=status[t.phase];
  $('thousand-table').append(node('p',`Deal ${t.deal||'—'} · Trump: ${t.trump?symbols[t.trump]:'none'}${t.contractor?` · ${name(t.contractor)}: ${t.order}${t.compulsory?' · compulsory':''}`:''}`,'table-meta'));
  if(t.stock){const box=node('div','','stock');box.append(node('h3',t.stockTaken?'Stock · taken':'Stock · public'),cardsRow(t.stock));$('thousand-table').append(box);}
  const trick=node('div','','trick-area');trick.setAttribute('aria-label','Current trick');
  for(const play of t.plays){const slot=node('div','','trick-play');slot.append(node('small',name(play.player)),cardsRow([play.card]));trick.append(slot);}
  if(!t.plays.length)trick.append(node('p',t.phase==='trick'?'The table is clear.':'','muted'));
  $('thousand-table').append(trick);
  if(t.closeAt){closureTimer=setTimeout(refresh,Math.max(30,t.closeAt-t.serverNow+50));}
  if(c.shuffle)$('table-actions').append(btn(t.deal?'Shuffle & deal next hand':'Shuffle & deal',{type:'shuffle'}));
  if(c.fourNines)$('table-actions').append(btn('Declare four nines',{type:'fourNines'},'secondary'));
  if(c.dismiss)$('table-actions').append(btn('Take a stick · палочка',{type:'dismiss'},'secondary'));
  if(c.pass)$('game-actions').append(btn('Pass',{type:'pass'},'secondary'));
  if(c.here)$('game-actions').append(btn(`Here · здесь · ${t.order}`,{type:'here'},'secondary'));
  function amountForm(text,min,max,value,action){
    const form=node('form','','inline-form');const input=node('input');input.type='number';input.min=String(min);input.max=String(max);input.step='5';input.required=true;input.value=String(draft.amount??value);input.oninput=()=>{draft.amount=input.value;};input.setAttribute('aria-label',text);
    const submit=node('button',text);submit.type='submit';form.append(input,submit);form.onsubmit=e=>{e.preventDefault();run(()=>send({type:action,amount:Number(input.value)}));};$('game-actions').append(form);
  }
  if(c.bidMin!==null)amountForm('Raise bid',c.bidMin,c.bidMax,c.bidMin,'bid');
  if(c.takeStock)$('game-actions').append(btn('Take stock',{type:'takeStock'}));
  if(c.recipients.length){
    const form=node('form','','distribution-form');form.append(node('p','Choose a different card for each opponent. Visibility is independent.'));
    const inputs=c.recipients.map((recipient,i)=>{
      const select=node('select');select.required=true;for(const card of t.hand){const option=node('option',cardName(card));option.value=card;select.append(option);}select.value=draft[recipient.id]?.card??t.hand[i];
      const publicInput=node('input');publicInput.type='checkbox';publicInput.checked=draft[recipient.id]?.public??false;
      const save=()=>{draft[recipient.id]={card:select.value,public:publicInput.checked};};select.onchange=save;publicInput.onchange=save;
      const row=node('div','','transfer-row');row.append(label(`Card for ${recipient.name}`,select),label('Give openly',publicInput));form.append(row);return {recipient,select,publicInput};
    });
    const submit=node('button','Give cards');submit.type='submit';form.append(submit);form.onsubmit=e=>{e.preventDefault();run(()=>send({type:'distribute',transfers:inputs.map(({recipient,select,publicInput})=>({player:recipient.id,card:select.value,public:publicInput.checked}))}));};$('game-actions').append(form);
  }
  if(c.order)amountForm('Set order & start',t.order,c.bidMax,t.order,'order');
  if(c.legalCards.length){
    const play=btn('Play selected card',{type:'play',card:selected,announce:false});play.disabled=!selected||!c.legalCards.includes(selected);$('game-actions').append(play);
    if(selected&&c.announceCards.includes(selected))$('game-actions').append(btn(`Play & announce ${symbols[selected.slice(-1)]} marriage`,{type:'play',card:selected,announce:true}));
  }
  if(t.transfers.length){const info=node('div','','transfer-history');info.append(node('h3','Card transfers you can see'));for(const transfer of t.transfers)info.append(node('p',`${cardName(transfer.card)} → ${name(transfer.player)} · ${transfer.public?'public':'private'}`));$('thousand-table').append(info);}
  $('trick-piles').replaceChildren();
  const heading=node('h3','Collected tricks · inspect each player’s last trick');$('trick-piles').append(heading);
  const piles=node('div','','pile-grid');
  for(const pile of t.piles){
    const details=node('details','','panel pile');details.open=openPiles.has(pile.player);details.ontoggle=()=>{if(details.open)openPiles.add(pile.player);else openPiles.delete(pile.player);};
    details.append(node('summary',`${name(pile.player)}${pile.player===me.id?' · you':''} · ${pile.count} tricks`));
    if(pile.top){details.append(node('p',`Trick ${pile.top.number} · ${pile.top.points} points · Trump: ${pile.top.trump?symbols[pile.top.trump]:'none'}`));for(const play of pile.top.plays){const row=node('div','','pile-play');row.append(node('span',name(play.player)),cardsRow([play.card]));details.append(row);}}
    else details.append(node('p','No tricks collected yet.'));
    piles.append(details);
  }
  $('trick-piles').append(piles);
  if(t.lastDeal&&['awaitShuffle','finished'].includes(t.phase)){
    const summary=node('div','','panel deal-summary');summary.append(node('h3',`Deal ${t.lastDeal.deal} · ${t.lastDeal.status}`));
    for(const player of room.players){const before=t.lastDeal.before[player.id],after=t.lastDeal.after[player.id],delta=after-before;summary.append(node('p',`${player.name}: ${before} → ${after} (${delta>=0?'+':''}${delta}) · sticks ${t.lastDeal.sticks[player.id]}`));}
    $('trick-piles').append(summary);
  }
  $('audit').disabled=t.auditDeals===0;
  $('audit').textContent=t.auditDeals?`Download audit · ${t.auditDeals} settled deal${t.auditDeals===1?'':'s'}`:'Audit available after the deal';
}
