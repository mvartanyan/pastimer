const SUITS = {
  H: { name: 'hearts', glyph: '♥', color: '#b63840', path: 'M12 21C9 18 1 13 1 7A5.5 5.5 0 0 1 12 5A5.5 5.5 0 0 1 23 7C23 13 15 18 12 21Z' },
  D: { name: 'diamonds', glyph: '♦', color: '#b63840', path: 'M12 0 23 12 12 24 1 12Z' },
  C: { name: 'clubs', glyph: '♣', color: '#172c38', path: 'M9 15C3 20-2 13 2 9C4 7 7 7 9 9C4 3 9-2 13 0C18 1 18 6 15 9C22 5 27 13 21 17C18 19 15 17 14 15L17 24H7L10 15Z' },
  S: { name: 'spades', glyph: '♠', color: '#172c38', path: 'M12 0C9 4 0 10 1 15C2 20 8 21 11 17L8 24H16L13 17C17 22 23 19 23 15C24 10 15 4 12 0Z' },
};
const RANKS = ['2','3','4','5','6','7','8','9','10','J','Q','K','A'];
const NAMES = { A:'Ace', K:'King', Q:'Queen', J:'Jack' };
let serial=0;
export const cardName=code=>code.slice(0,-1)+(SUITS[code.slice(-1)]?.glyph??'');
export const cardLabel=code=>`${NAMES[code.slice(0,-1)]??code.slice(0,-1)} of ${SUITS[code.slice(-1)]?.name??'cards'}`;
const pip=(s,x,y,size=21,rotate=false)=>`<g transform="translate(${x} ${y})${rotate?' rotate(180)':''}"><path d="${s.path}" fill="${s.color}" transform="translate(${-size/2} ${-size/2}) scale(${size/24})"/></g>`;
function court(rank,s){
  const gold='#ba954f',navy='#24495b',red='#a23b42',skin='#f1d6aa';
  const head=rank==='J'?`<path d="M57 42Q77 24 99 42L101 48H56Z" fill="${red}"/><path d="M88 35Q108 13 107 36L96 41" fill="${gold}"/>`:`<path d="M57 43 54 27 67 34 78 21 89 34 103 27 100 43Z" fill="${gold}" stroke="${navy}" stroke-width="1.5"/><path d="M57 43H101V49H57Z" fill="${red}"/>`;
  const face=rank==='K'?`<path d="M65 72Q79 88 93 72L90 89 79 96 68 89Z" fill="${navy}"/><path d="M68 70Q74 65 79 71Q85 65 91 70" fill="none" stroke="${navy}" stroke-width="3"/>`:rank==='Q'?`<path d="M62 49Q49 59 58 89L68 82V51M97 49Q110 59 101 89L91 82V51" fill="${navy}"/><circle cx="61" cy="72" r="3" fill="${gold}"/><circle cx="98" cy="72" r="3" fill="${gold}"/>`:`<path d="M64 48 58 64 64 70M96 48 102 64 96 70" stroke="${navy}" stroke-width="5" fill="none"/>`;
  const accessory=rank==='Q'?`<path d="M48 108 50 74" stroke="${gold}" stroke-width="2"/><path d="M50 82Q36 75 42 67Q45 62 50 69Q57 60 61 67Q64 75 50 82" fill="${red}" stroke="${gold}"/><path d="M50 95Q38 94 41 85Q50 85 50 95" fill="${navy}"/>`:`<path d="M${rank==='K'?'110 107 110 38 115 29 120 38 120 107':'46 109 46 37 50 28 54 37 54 109'}" fill="${gold}" stroke="${navy}" stroke-width="1.5"/><path d="M${rank==='K'?'105 82H125':'40 82H60'}" stroke="${red}" stroke-width="4"/>`;
  const half=`<path d="M43 111V94L63 79H96L117 94V111Z" fill="${navy}"/><path d="M45 97 70 83 101 111H75Z" fill="${red}"/><path d="M48 96 76 112M62 86 91 112M92 84 111 108" stroke="${gold}" stroke-width="3"/><path d="M62 47Q79 41 97 47L94 73 79 83 65 73Z" fill="${skin}" stroke="${navy}" stroke-width="1.4"/>${face}<path d="M68 55H74M85 55H91" stroke="${navy}" stroke-width="2"/><path d="M79 56 77 65H82M74 72H84" stroke="${red}" fill="none" stroke-width="1.2"/>${head}${accessory}<path d="M72 95 80 87 88 95 80 103Z" fill="${gold}"/><path d="M80 92 83 95 80 98 77 95Z" fill="${red}"/>`;
  return `<rect x="37" y="24" width="86" height="176" rx="2" fill="#ede2c9" stroke="${gold}" stroke-width="1.4"/><path d="M40 111H120M40 113H120" stroke="${gold}"/>${half}<g transform="translate(160 224) rotate(180)">${half}</g><circle cx="80" cy="112" r="24" fill="#fff8e9" stroke="${gold}" stroke-width="2"/>${pip(s,80,112,38)}`;
}
function art(code){
  const rank=code.slice(0,-1),s=SUITS[code.slice(-1)];
  if(!s||!RANKS.includes(rank))throw new Error('Unknown playing card');
  const id=`paper-${++serial}`;
  let center='';
  if(['K','Q','J'].includes(rank))center=court(rank,s);
  else if(rank==='A')center=`${pip(s,80,107,rank==='A'&&code.endsWith('S')?62:54)}<path d="M60 154H100M67 158H93" stroke="#c4aa74" stroke-width="1"/>`;
  else{
    const n=Number(rank),positions=[];
    if(n===2)positions.push([80,56],[80,168]);
    else if(n===3)positions.push([80,54],[80,112],[80,170]);
    else{
      const rows=n>=8?[48,89,135,176]:n>=6?[52,112,172]:[56,168];
      for(const y of rows){positions.push([53,y],[107,y]);}
      if(n===5||n===9)positions.push([80,112]);
      if(n===7)positions.push([80,81]);
      if(n===10)positions.push([80,69],[80,155]);
    }
    center=positions.map(([x,y])=>pip(s,x,y,n>=8?26:29,y>112)).join('');
  }
  const corner=`<text x="22" y="36" text-anchor="middle" fill="${s.color}" font-family="Georgia,serif" font-size="${rank==='10'?30:36}" font-weight="bold">${rank}</text>${pip(s,22,63,32)}`;
  center=`<g transform="translate(5 0) scale(.95 1)">${center}</g>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 224" aria-hidden="true" focusable="false"><defs><linearGradient id="${id}" x2="1" y2="1"><stop stop-color="#fffdf6"/><stop offset="1" stop-color="#f1e8d3"/></linearGradient></defs><rect x=".75" y=".75" width="158.5" height="222.5" rx="10" fill="url(#${id})" stroke="#d6cbb4" stroke-width="1.5"/>${corner}<g transform="translate(160 224) rotate(180)">${corner}</g>${center}</svg>`;
}
export function playingCard(code,{interactive=false,selected=false,className=''}={}){
  const e=document.createElement(interactive?'button':'span');
  e.className=`playing-card ${interactive?'card ':''}${selected?'selected ':''}${className}`.trim();
  e.dataset.card=code;e.setAttribute('aria-label',cardLabel(code));
  if(interactive){e.type='button';e.setAttribute('aria-pressed',String(selected));}
  else e.setAttribute('role','img');
  e.innerHTML=art(code);return e;
}
export function cardBack(className=''){
  const e=document.createElement('span');e.className=`card-back ${className}`;e.setAttribute('aria-hidden','true');
  e.innerHTML='<span class="back-ornament">♠</span>';return e;
}
export function sortedHand(cards,mode='suit',game='thousand'){
  const suits=['S','H','C','D'];
  const order=game==='thousand'?['2','3','4','5','6','7','8','9','J','Q','K','10','A']:RANKS;
  const rank=card=>order.indexOf(card.slice(0,-1));
  return [...cards].sort((a,b)=>mode==='rank'?rank(b)-rank(a)||suits.indexOf(a.slice(-1))-suits.indexOf(b.slice(-1)):suits.indexOf(a.slice(-1))-suits.indexOf(b.slice(-1))||rank(b)-rank(a));
}
export function layoutHand(){
  const hand=document.getElementById('hand');if(!hand?.children.length||hand.classList.contains('durak-hand'))return;
  const n=hand.children.length,available=Math.max(120,hand.clientWidth-8),height=Math.max(60,hand.clientHeight-8),width=Math.min(112,height*5/7,available);
  const overlap=n>1?Math.max(0,(n*width-available)/(n-1)):0;
  hand.style.setProperty('--card-width',`${width}px`);hand.parentElement.style.setProperty('--card-width',`${width}px`);hand.style.setProperty('--overlap',`${overlap}px`);
  for(const [i,card]of [...hand.children].entries()){card.style.setProperty('--tilt','0deg');card.style.setProperty('--arc','0px');card.style.setProperty('--card-index',String(i));}
}
