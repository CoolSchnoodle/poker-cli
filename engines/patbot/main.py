#!/usr/bin/env python3
"""PatBot - a Texas Hold'em engine for the Programming Club Poker Bot League.

Pure Python 3 standard library (no pip packages, no network), so it runs on
any league machine after `build.sh`.

Interface (see player_api.md):
  run.sh START <tmpdir>            -> prints only the engine name
  run.sh <min_bet> <turn_ms> <bank_ms> <params> <state> <actions> <tmpdir>
                                   -> prints exactly one of FOLD / CALL / RAISE <n>

Interpretation notes (the spec leaves these implicit):
  * min_bet is the TOTAL bet a player must have in the pot to stay in
    (bets[] in the game state are totals), so calling costs
    max(0, min_bet - my_bet).
  * RAISE <n> is likewise a TOTAL bet target, not an amount to add.
  * CALL when min_bet <= my_bet is a free check.

Strategy in brief:
  * Preflop: a calibrated hand-strength estimate (pairs + Chen-style score
    mapped to equity vs one random hand).
  * Postflop: Monte Carlo equity vs one random opponent hand (exact
    enumeration on the river), adjusted for the number of live opponents.
  * Pot-odds-aware calls, value raises sized off the pot, position-aware
    thresholds, and a push/fold mode when short-stacked.
  * Every decision is defensive: any bad/missing input degrades to a legal
    FOLD (or a free CALL), never a crash and never extra stdout.
"""

import json
import os
import random
import sys
import time

ENGINE_NAME = "PatBot"

# --------------------------------------------------------------------------
# Cards: an int 0..51, rank = c // 4 + 2 (2..14, 14 = Ace), suit = c % 4.
# --------------------------------------------------------------------------
RANKS = [c // 4 + 2 for c in range(52)]
SUITS = [c % 4 for c in range(52)]

_RNG = random.Random((time.time_ns() ^ (os.getpid() << 16)) & 0xFFFFFFFFFFFFFFFF)


def _card(rank, suit):
    return (rank - 2) * 4 + suit


# --------------------------------------------------------------------------
# 7-card hand evaluation. Returns a single comparable int; higher is better.
# Category 8 = straight flush ... 0 = high card, kickers packed base-15.
# --------------------------------------------------------------------------
def _score(cat, kickers):
    s = cat
    for i in range(5):
        s = s * 15 + (kickers[i] if i < len(kickers) else 0)
    return s


def _straight_high(present):
    """Highest card of a 5-card straight in a set of ranks, else 0."""
    for high in range(14, 4, -1):
        if all((high - i) in present for i in range(5)):
            return high
    if {14, 5, 4, 3, 2} <= present:
        return 5
    return 0


def eval7(cards):
    rank_count = [0] * 15
    suit_count = [0] * 4
    suit_ranks = ([], [], [], [])
    for c in cards:
        r = RANKS[c]
        s = SUITS[c]
        rank_count[r] += 1
        suit_count[s] += 1
        suit_ranks[s].append(r)

    flush_suit = -1
    for s in range(4):
        if suit_count[s] >= 5:
            flush_suit = s
            break
    if flush_suit >= 0:
        fr = sorted(suit_ranks[flush_suit], reverse=True)
        sf = _straight_high(set(fr))
        if sf:
            return _score(8, [sf])
        return _score(5, fr[:5])

    present = {r for r in range(2, 15) if rank_count[r]}
    straight = _straight_high(present)

    # groups: (count, rank), sorted by count desc then rank desc
    groups = sorted(
        ((rank_count[r], r) for r in range(14, 1, -1) if rank_count[r]),
        key=lambda t: (t[0], t[1]),
        reverse=True,
    )
    pairs = [r for cnt, r in groups if cnt == 2]
    trips = [r for cnt, r in groups if cnt == 3]

    def rest(excluded):
        return [r for r in range(14, 1, -1)
                if rank_count[r] and r not in excluded]

    if groups[0][0] == 4:
        return _score(7, [groups[0][1], rest({groups[0][1]})[0]])
    if groups[0][0] == 3 and (pairs or len(trips) >= 2):
        # Full house: best trip + best remaining pair (or second trip).
        pair_rank = pairs[0] if pairs else trips[1]
        return _score(6, [trips[0], pair_rank])
    if straight:
        return _score(4, [straight])
    if groups[0][0] == 3:
        return _score(3, [trips[0]] + rest({trips[0]})[:2])
    if len(pairs) >= 2:
        return _score(2, [pairs[0], pairs[1],
                          rest({pairs[0], pairs[1]})[0]])
    if len(pairs) == 1:
        return _score(1, [pairs[0]] + rest({pairs[0]})[:3])
    ranks_desc = [r for r in range(14, 1, -1) if rank_count[r]]
    return _score(0, ranks_desc[:5])


# --------------------------------------------------------------------------
# Equity estimation
# --------------------------------------------------------------------------
def preflop_equity(hole):
    """Estimated equity of a 2-card hand vs one random hand (0..1).

    Pairs use a linear fit to known equities; unpaired hands use a
    Chen-style score (high card + suited/connected bonuses, gap penalties)
    mapped linearly. Anchors: AA ~0.85, AKs ~0.67, AKo ~0.64, 72o ~0.35.
    """
    r1, r2 = sorted((RANKS[hole[0]], RANKS[hole[1]]), reverse=True)
    if r1 == r2:
        return min(0.87, 0.50 + 0.029 * (r1 - 2))
    suited = SUITS[hole[0]] == SUITS[hole[1]]
    high_val = {14: 10.0, 13: 8.0, 12: 7.0, 11: 6.0}.get(r1, r1 / 2.0)
    score = high_val
    if suited:
        score += 1.5
    gap = r1 - r2
    if gap == 1:
        score -= 1.0
    elif gap == 2:
        score -= 2.0
    elif gap == 3:
        score -= 4.0
    elif gap >= 4:
        score -= 5.0
    if gap <= 1 and r1 < 14:
        score += 0.5
    return min(0.87, max(0.30, 0.35 + 0.030 * max(score, 0.0)))


def mc_equity(hole, board, iters, deadline):
    """Monte Carlo equity vs one random opponent hand. Stops at deadline."""
    known = set(hole) | set(board)
    deck = [c for c in range(52) if c not in known]
    need = 5 - len(board)
    wins = 0.0
    n = 0
    for i in range(iters):
        if (i & 63) == 0 and time.monotonic() > deadline:
            break
        s = _RNG.sample(deck, 2 + need)
        opp = s[0:2]
        full_board = board + s[2:]
        mine = eval7(hole + full_board)
        theirs = eval7(opp + full_board)
        if mine > theirs:
            wins += 1.0
        elif mine == theirs:
            wins += 0.5
        n += 1
    return (wins / n) if n else 0.5


def river_equity(hole, board):
    """Exact equity vs one random hand on the river (all 990 combos)."""
    known = set(hole) | set(board)
    deck = [c for c in range(52) if c not in known]
    mine = eval7(hole + board)
    wins = 0.0
    total = 0
    for i in range(len(deck)):
        for j in range(i + 1, len(deck)):
            theirs = eval7([deck[i], deck[j]] + board)
            if mine > theirs:
                wins += 1.0
            elif mine == theirs:
                wins += 0.5
            total += 1
    return (wins / total) if total else 0.5


# --------------------------------------------------------------------------
# Decision
# --------------------------------------------------------------------------
def _safe_int(value, default=0):
    try:
        return int(value)
    except (TypeError, ValueError):
        return default


def decide(params, state, min_bet, turn_time_ms):
    n = len(state.get("funds") or [0])
    you = _safe_int(state.get("you_index"), 0)
    if you < 0 or you >= n:
        you = 0
    funds = list(state.get("funds") or [0] * n)
    bets = list(state.get("bets") or [0] * n)
    folds = list(state.get("folds") or [False] * n)
    # pad/truncate defensively
    funds += [0] * (n - len(funds))
    bets += [0] * (n - len(bets))
    folds += [False] * (n - len(folds))
    funds, bets, folds = funds[:n], bets[:n], folds[:n]

    my_funds = max(0, _safe_int(funds[you]))
    my_bet = max(0, _safe_int(bets[you]))
    stack = my_funds + my_bet  # everything I control this hand

    bb = max(1, _safe_int(params.get("big_blind"), 20))
    ante = max(0, _safe_int(params.get("ante"), 0))

    hand = state.get("your_hand") or []
    if len(hand) != 2:
        # Cannot evaluate: take a free check if offered, else fold.
        return "CALL" if min_bet <= my_bet else "FOLD"
    try:
        hole = [_card(int(hand[0][0]), int(hand[0][1])),
                _card(int(hand[1][0]), int(hand[1][1]))]
    except (TypeError, ValueError, IndexError):
        return "CALL" if min_bet <= my_bet else "FOLD"
    if hole[0] == hole[1]:
        return "CALL" if min_bet <= my_bet else "FOLD"

    board = []
    for c in state.get("revealed_cards") or []:
        try:
            board.append(_card(int(c[0]), int(c[1])))
        except (TypeError, ValueError, IndexError):
            continue
    board = [c for c in board if c not in hole][:5]
    street = len(board)  # 0 preflop, 3 flop, 4 turn, 5 river

    pot = sum(max(0, _safe_int(b)) for b in bets)
    if street == 0 and ante:
        pot += ante * n

    # Live opponents: not folded, still have chips in play.
    live_opps = [i for i in range(n)
                 if i != you and not folds[i]
                 and (max(0, _safe_int(funds[i])) > 0 or max(0, _safe_int(bets[i])) > 0)]
    n_opp = len(live_opps)
    if n_opp == 0:
        return "CALL"  # nobody left to beat; nothing to do

    # Position: anyone still able to act after me?
    order_after = [(you + k) % n for k in range(1, n)]
    acts_after = [i for i in order_after
                  if not folds[i] and max(0, _safe_int(funds[i])) > 0]
    is_last = len(acts_after) == 0

    call_cost = max(0, min_bet - my_bet)
    pot_after_call = pot + call_cost
    pot_odds = (call_cost / pot_after_call) if pot_after_call > 0 else 0.0
    stack_bb = stack / float(bb)

    # --- equity -----------------------------------------------------------
    if street == 0:
        eq1 = preflop_equity(hole)
    else:
        # Time budget for simulation: at most ~60% of the turn time,
        # never more than 600 ms, never less than 40 ms of sims.
        budget_ms = turn_time_ms if turn_time_ms and turn_time_ms > 0 else 1000
        sim_ms = max(40.0, min(600.0, 0.6 * budget_ms))
        deadline = time.monotonic() + sim_ms / 1000.0
        if street == 5:
            eq1 = river_equity(hole, board)
        else:
            iters = 350 if street == 4 else 300
            if budget_ms < 500:
                iters = max(60, int(iters * budget_ms / 500.0))
            eq1 = mc_equity(hole, board, iters, deadline)
    # Rough multi-way adjustment: chance of beating every live opponent.
    eq = eq1 ** n_opp if n_opp > 1 else eq1
    eq_raw = eq  # pre-discount: used for shove decisions (fold equity helps)
    if street == 0 and min_bet > bb:
        # Facing a preflop raise, the raiser's range is far stronger than
        # a random hand, so discount the vs-random equity. Without this,
        # junk like 72o looks callable on raw pot odds alone.
        eq *= 0.88

    # --- thresholds -------------------------------------------------------
    pos_adj = -0.02 if is_last else 0.0  # looser in position
    if street == 0:
        pos_adj += -0.02 if n_opp == 1 else 0.0  # heads-up: play wide
    raise_eq = {0: 0.66, 3: 0.62, 4: 0.65, 5: 0.70}[street] + pos_adj
    call_margin = 0.03 + pos_adj
    if street in (3, 4) and eq >= 0.28:
        call_margin -= 0.04  # implied odds: draws can call a bit looser

    def raise_to():
        """Total-bet target for a value raise, capped at my stack."""
        if street == 0:
            if min_bet <= bb:
                limpers = sum(1 for i in live_opps
                              if 0 < max(0, _safe_int(bets[i])) <= bb)
                target = (3 + limpers) * bb
            else:
                target = max(min_bet * 3, min_bet + 3 * bb)
        else:
            sizing = 0.75 * pot
            if eq >= 0.85:
                sizing = 1.0 * pot
            target = min_bet + max(bb, int(round(sizing)))
        return min(max(target, min_bet + 1), stack)

    # --- short stack: push / fold ----------------------------------------
    if stack_bb <= 12 and stack > 0:
        shove_eq = 0.50 if stack_bb <= 6 else 0.55
        if eq_raw >= shove_eq and call_cost < stack:
            return "RAISE %d" % stack  # all-in
        if call_cost == 0:
            return "CALL"
        if eq > pot_odds + 0.05:
            return "CALL"
        return "FOLD"

    # --- already all-in: nothing to decide --------------------------------
    if my_funds <= 0:
        return "CALL"

    # --- main decision ----------------------------------------------------
    if call_cost == 0:
        # Free action: raise for value, else check.
        if eq >= raise_eq:
            target = raise_to()
            if target > min_bet:
                return "RAISE %d" % target
        return "CALL"

    if eq >= raise_eq:
        target = raise_to()
        if target > min_bet:
            return "RAISE %d" % target
        return "CALL"
    if eq > pot_odds + call_margin:
        return "CALL"
    return "FOLD"


def main(argv):
    if len(argv) < 2:
        print("FOLD")
        return 0
    if argv[1] == "START":
        print(ENGINE_NAME)
        return 0
    if len(argv) < 8:
        print("FOLD")
        return 0

    min_bet = _safe_int(argv[1], 0)
    turn_time_ms = _safe_int(argv[2], 1000)
    # argv[3] = time bank, argv[6] = actions file, argv[7] = temp dir:
    # read but not needed for decisions.
    try:
        with open(argv[4], "r") as f:
            params = json.load(f)
    except Exception:
        params = {}
    try:
        with open(argv[5], "r") as f:
            state = json.load(f)
    except Exception:
        state = {}
    try:
        action = decide(params, state, min_bet, turn_time_ms)
    except Exception as exc:  # never crash, never print junk to stdout
        print("decision error: %r" % (exc,), file=sys.stderr)
        action = "CALL" if min_bet <= 0 else "FOLD"
    # Final sanity: exactly one legal action on stdout.
    if action.startswith("RAISE"):
        try:
            amt = int(action.split()[1])
            if amt <= 0:
                raise ValueError
            print("RAISE %d" % amt)
        except (IndexError, ValueError):
            print("CALL" if min_bet <= 0 else "FOLD")
    elif action in ("FOLD", "CALL"):
        print(action)
    else:
        print("CALL" if min_bet <= 0 else "FOLD")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
