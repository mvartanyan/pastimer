use super::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub kind: &'static str,
    pub phase: &'static str,
    pub deal: u32,
    pub dealer: String,
    pub contractor: Option<String>,
    pub order: u16,
    pub compulsory: bool,
    pub turn: Option<String>,
    pub hand: Vec<Card>,
    pub stock: Option<Vec<Card>>,
    pub stock_taken: bool,
    pub trump: Option<Suit>,
    pub seats: Vec<SeatView>,
    pub piles: Vec<PileView>,
    pub plays: Vec<Play>,
    pub close_at: Option<i64>,
    pub server_now: i64,
    pub controls: Controls,
    pub transfers: Vec<Transfer>,
    pub last_deal: Option<Value>,
    pub audit_deals: usize,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    #[serde(flatten)]
    pub player: Player,
    pub count: usize,
    pub score: i64,
    pub sticks: u32,
    pub on_barrel: bool,
    pub round_points: u16,
    pub role: &'static str,
}
#[derive(Serialize)]
pub struct PileView {
    pub player: String,
    pub count: usize,
    pub top: Option<Trick>,
}
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Controls {
    pub shuffle: bool,
    pub pass: bool,
    pub here: bool,
    pub bid_min: Option<u16>,
    pub bid_max: u16,
    pub take_stock: bool,
    pub recipients: Vec<Player>,
    pub order: bool,
    pub dismiss: bool,
    pub four_nines: bool,
    pub legal_cards: Vec<Card>,
    pub announce_cards: Vec<Card>,
}
impl State {
    pub fn view(&self, viewer: &str, players: &[Player], now: i64) -> View {
        let actor = players.iter().position(|p| p.id == viewer);
        let mut controls = Controls {
            bid_max: MAX_ORDER,
            ..Controls::default()
        };
        controls.shuffle = matches!(self.phase, Phase::AwaitShuffle) && actor == Some(self.dealer);
        controls.four_nines = !matches!(self.phase, Phase::AwaitShuffle | Phase::Finished)
            && self
                .hands
                .get(viewer)
                .is_some_and(|h| h.iter().filter(|c| c.face() == "9").count() == 4);
        let mut turn = None;
        match &self.phase {
            Phase::Auction(a) => {
                turn = players.get(a.turn).map(|p| p.id.clone());
                if actor == Some(a.turn) {
                    controls.pass = true;
                    controls.here = a.last_pass;
                    if self.order + 10 <= MAX_ORDER {
                        controls.bid_min = Some(self.order + 10);
                    }
                }
            }
            Phase::Stock => {
                controls.take_stock = actor == self.contractor;
            }
            Phase::Distribution => {
                if actor == self.contractor {
                    controls.recipients = self
                        .active
                        .iter()
                        .filter(|p| Some(**p) != actor)
                        .map(|p| players[*p].clone())
                        .collect();
                }
            }
            Phase::Preparation => {
                controls.order = actor == self.contractor;
            }
            Phase::Trick { turn: t, .. } => {
                turn = players.get(*t).map(|p| p.id.clone());
                if actor == Some(*t) {
                    controls.legal_cards = self.legal_cards(viewer);
                    controls.announce_cards = controls
                        .legal_cards
                        .iter()
                        .filter(|c| self.can_announce(viewer, c))
                        .cloned()
                        .collect();
                }
            }
            _ => {}
        }
        controls.dismiss = matches!(
            self.phase,
            Phase::Stock | Phase::Distribution | Phase::Preparation
        ) && actor == self.contractor
            && !self.compulsory;
        let (plays, close_at) = match &self.phase {
            Phase::Trick { plays, .. } => (plays.clone(), None),
            Phase::Closing { trick, close_at } => (trick.plays.clone(), Some(*close_at)),
            _ => (vec![], None),
        };
        View{kind:"thousand",phase:self.phase.name(),deal:self.deal,dealer:players.get(self.dealer).map(|p|p.id.clone()).unwrap_or_default(),contractor:self.contractor.and_then(|p|players.get(p)).map(|p|p.id.clone()),order:self.order,compulsory:self.compulsory,
            turn,hand:self.hands.get(viewer).cloned().unwrap_or_default(),stock:self.stock_visible.then(||self.stock.clone()),stock_taken:self.stock_taken,trump:self.trump,
            seats:players.iter().enumerate().map(|(i,p)|SeatView{player:p.clone(),count:self.hands.get(&p.id).map_or(0,Vec::len),score:self.ledger.get(&p.id),sticks:self.sticks.get(&p.id).copied().unwrap_or(0),on_barrel:self.ledger.get(&p.id)==i64::from(self.settings.barrel_threshold),round_points:self.raw.get(&p.id).copied().unwrap_or(0),role:if players.len()==4&&i==self.dealer{"dealer"}else if self.contractor==Some(i){"contractor"}else{"player"}}).collect(),
            piles:players.iter().map(|p|PileView{player:p.id.clone(),count:self.tricks.iter().filter(|t|t.winner==p.id).count(),top:self.tricks.iter().rev().find(|t|t.winner==p.id).cloned()}).collect(),plays,close_at,server_now:now,controls,
            transfers:self.movements.iter().filter_map(|m|{if let (Zone::Hand{player:from},Zone::Hand{player:to})=(&m.from,&m.to)&& (m.public||from==viewer||to==viewer){return Some(Transfer{player:to.clone(),card:m.card.clone(),public:m.public})}None}).collect(),
            last_deal:self.archives.last().map(|r|json!({"deal":r.deal,"status":r.status,"contract":r.contract,"before":r.before.totals,"after":r.after.totals,"sticks":r.sticks_after})),audit_deals:self.archives.len()}
    }
}
