use crate::{
    model::{Action, GameState, Lifecycle, Room},
    thousand::{self, Transfer, View},
};

/// Deliberately simple policy using only this seat's player-visible information.
fn choose(view: View) -> Option<thousand::Action> {
    let controls = view.controls;
    if controls.pass {
        return Some(thousand::Action::Pass);
    }
    if controls.take_stock {
        return Some(thousand::Action::TakeStock);
    }
    if controls.recipients.len() == 2 {
        let mut hand = view.hand;
        hand.sort_by_key(thousand::rank);
        if hand.len() < 2 {
            return None;
        }
        let transfers = controls
            .recipients
            .into_iter()
            .zip(hand)
            .map(|(player, card)| Transfer {
                player: player.id,
                card,
                public: false,
            })
            .collect();
        return Some(thousand::Action::Distribute { transfers });
    }
    if controls.order {
        return Some(thousand::Action::Order { amount: view.order });
    }
    let card = controls
        .announce_cards
        .first()
        .cloned()
        .or_else(|| controls.legal_cards.into_iter().min_by_key(thousand::rank))?;
    let announce = controls.announce_cards.contains(&card);
    Some(thousand::Action::Play { card, announce })
}

pub fn next(room: &Room, now: i64) -> Option<(String, Action)> {
    let Lifecycle::Playing(GameState::Thousand(state)) = &room.lifecycle else {
        return None;
    };
    // Shuffling is intentionally requested by a human at each deal boundary.
    // Everything else stops at the next human turn or the persisted trick deadline.
    room.computers.iter().find_map(|id| {
        choose(state.view(id, &room.players, now))
            .map(|action| (id.clone(), Action::Thousand { action }))
    })
}
