# Thousand / тысяча — owner's variant

One **deal** is a раздача, one **trick** is a взятка, and a **match** is play toward 1000.

## Cards and seats

- Twenty-four cards: A, 10, K, Q, J, 9 in each suit.
- Trick strength, highest first: A > 10 > K > Q > J > 9.
- Card points respectively: 10, 10, 5, 5, 5, 0; 140 card points per deck.
- Suits have equal standing except for the current trump suit.
- Marriage values: hearts 100, diamonds 80, clubs 60, spades 40.
- Three or four players; exactly three play each deal. Each receives seven cards; three cards form the stock.
- Seats follow joining order. The first dealer is chosen randomly; dealing, bidding and play follow that circular seating order. The dealer advances after each completed or dismissed deal.
- With four players the dealer sits out, determines the deck randomization, and, unless on the barrel, receives 10 points per ace in the stock plus half the value of a marriage in the stock. The bonuses are additive.
- With three players there is no dealer stock bonus.

## Auction and stock

- The player after the dealer posts compulsory 100.
- Each raise must be at least 10 above the current bid; bids must be divisible by 5. From 100, 110 and 115 are valid; 105 is not.
- A marriage in hand is not required to bid. The maximum bid and final order are 380: 140 card points plus the three highest marriages, worth 240. A player cannot announce all four marriages from an eight-card hand: holding all four leaves only kings and queens, and the first card they play breaks a pair before they can announce it.
- Passing removes the player from that auction.
- A player immediately following a pass may say “here” to take the current bid without raising. Example: A bids 100, B passes, C says here at 100; A may then raise or pass, and B remains out.
- If the original compulsory bidder subsequently wins the contract back, that contract is no longer compulsory.
- If no active player is on the barrel, the stock stays hidden until the auction finishes, then is shown to everyone before the winner takes it. If any active player is on the barrel, the stock is dealt face up from the start. A sitting-out dealer on the barrel alone does not make the stock open.
- The winner gives one card to each other active player, choosing from the whole resulting hand, not just the stock. Each transfer may independently be public or secret, so one card may be given openly and the other privately. Each active player then has eight cards.
- After taking the stock and distributing, the winner may leave the order unchanged or increase it in steps of five, up to 380, before making the opening lead. For example, an order of 110 may become 115; the auction's minimum increase of 10 does not apply at this stage.
- Increasing an otherwise untouched compulsory 100 after taking the stock does not remove the obligation to play: the contractor still cannot take a stick.

## Tricks and marriages

- Follow the led suit if possible. Otherwise play trump if possible. Otherwise any card is legal.
- There is no requirement to beat the currently winning card.
- There is no trump until the first marriage is announced. The highest trump wins a trick containing trumps; otherwise the highest card of the led suit wins. The winner leads the next trick.
- After the third card is played, the completed trick remains visible for three seconds, then closes and joins the winner's personal pile of collected tricks.
- Every player has a separate trick pile for the current deal. Any player, including the sitting-out dealer, may inspect the top trick of any player's pile: the most recent trick that particular player won, including their own. A player who has not won a trick has an empty pile. Piles start empty on each new deal or redeal.
- A marriage may be announced by leading its king or queen while still holding the partner, after winning a trick. It cannot be announced on the opening lead of the deal. Announcement is optional: leading an eligible king or queen does not automatically announce it.
- Each announced marriage sets its suit as trump, replacing any previous trump.
- A legally announced marriage counts even if the announcing player loses that trick.

## Scores and dismissal

- The contractor succeeds when captured card points plus announced marriage points meet or exceed the final order. Success adds exactly the order to their match score; failure deducts exactly the order, irrespective of the excess or shortfall.
- Each defender scores their own captured card points plus their announced marriage points, independently of the contractor's result, subject to the barrel restriction on passive score gains.
- Compulsory contracts cannot be dismissed. All barrel contracts are compulsory, including contracts won through a barrel auction.
- Outside barrel play, non-compulsory contracts, including a “here” contract at 100, may be dismissed for a stick only before the opening lead. Inspecting the stock and distributing cards may precede dismissal.
- On dismissal the other two active players receive 50 each, subject to the barrel restriction on passive score gains; the contractor receives a stick. An eligible fourth-player dealer retains the stock bonus; a dealer already on the barrel receives none.
- Dismissal causes no other change to the contractor's score. Every third stick incurs a -150 penalty: sticks 3, 6, 9, etc. Their score is unchanged on the intervening sticks.
- There are no additional bonuses, penalties, or redeal conditions: no zero-trick penalty, last-trick bonus, ace marriage, or limit on barrel attempts.

## Barrel and victory

- The barrel threshold is chosen when creating the match: any multiple of five from 620 through 900, defaulting to 900. The lower bound ensures that the required barrel contract does not exceed 380.
- When a player below the barrel reaches or crosses its threshold, their score is clamped to the threshold: 890 + 40 becomes 900, not 930.
- Once on the barrel, only the player's own played contract changes their score. They receive no dealer stock bonus or defender points, including points for marriages. Reaching or exceeding 1000 to win requires making their own contract. Failure deducts the order; if the result is below the barrel threshold, they leave the barrel, resume ordinary scoring, and must reach the threshold again to re-enter it. For example, failing 100 at 900 leaves the player at 800 and off the barrel.
- Barrel participation is determined among the three active players; a sitting-out fourth-player dealer does not participate in the barrel contract or auction.
- With no active barrel players, use an ordinary auction, even if the sitting-out dealer is on the barrel.
- With exactly one active barrel player, that player plays a mandatory contract of 1000 minus the threshold without bidding. Success reaches or exceeds 1000 and wins; failure deducts the contract amount. A second barrel player sitting out as dealer does not change this rule.
- With multiple active barrel players, bidding is limited to those players. The first eligible player after the dealer posts the opening bid of 1000 minus the configured barrel threshold; subsequent turns skip players who are not on the barrel. For a threshold of 800 the auction starts at 200; for 900 it starts at 100. Otherwise normal auction rules apply, but the resulting contract is always compulsory and cannot be dismissed for a stick.
- Confirmed four-player example: A and B are both at 900, A deals and sits out, B/C/D play; B plays compulsory 100 without bidding. A receives no stock bonus and remains at 900.

## Four nines and cancellation

- An active player may declare four nines at any moment while still holding all four, including after taking the stock before distribution, or later in play. Waiting strategically is allowed.
- A valid declaration cancels the entire deal, including all score effects and dealer stock bonuses.
- Restart with a fresh shuffle, the same dealer and the same compulsory bidder.

## Online play and verification

- The shuffled deck's hash is published before dealing, and cards are then dealt without waiting for player acknowledgement. Every new shuffle, including a four-nines redeal, has its own published hash.
- Moves and cancellations remain in the game record. A cancelled deal's score effects are reversed without deleting its history.
- After each completed or cancelled deal, its deck and hidden transfers are revealed for verification; the following deal uses an independent shuffle.
- Each deal's audit package contains the initial shuffled deck in order and the data needed to check its published hash; the ordered record of every card movement, identifying the cards, their source and destination, and the player responsible where applicable; and every trick, including play order, the trump in force and the winner. Card movements include the initial deal, placement into and pickup of the stock, transfers between players, cards played to the table and collection of completed tricks into their winners' piles.
- If a player disconnects, their seat and hand are preserved. The table waits whenever their action is required; there are no automatic moves, timeout penalties or forfeits.
