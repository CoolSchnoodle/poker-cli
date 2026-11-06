# PatBot — Poker Bot League engine

A Texas Hold'em bot for the Programming Club Poker Bot League, written in
pure Python 3 (standard library only — no pip packages, no network), so it
runs on any league machine after `build.sh`.

**Submission:** this directory drops into
[CoolSchnoodle/poker-cli](https://github.com/CoolSchnoodle/poker-cli) as
`engines/patbot/` — the three files `build.sh`, `run.sh`, `main.py` are the
whole submission (plus this README). The engine name printed on `START`
is `PatBot`.

## Files

- `main.py` — the whole engine: hand evaluation, equity estimation, decisions.
- `build.sh` — byte-compiles `main.py` (catches syntax errors up front).
- `run.sh` — invokes `main.py` with the league's arguments.
- `harness/` — a local test harness (not part of the league submission):
  a Texas Hold'em game runner, three baseline bots, edge-case tests, and
  match results. Run `python3 harness/harness.py --help`.

## How it plays

- **Preflop:** calibrated hand-strength estimate (pairs fitted to known
  equities; unpaired hands via a Chen-style score mapped to equity vs one
  random hand).
- **Postflop:** Monte Carlo equity vs a random opponent hand (300 sims on
  the flop, 350 on the turn, time-boxed to ~60% of the turn time), and
  exact enumeration of all 990 opponent hands on the river.
- **Multi-way:** equity is adjusted for the number of live opponents
  (roughly, the chance of beating all of them).
- **Decisions:** pot-odds-aware calls (with an implied-odds allowance for
  draws), value raises sized at ~3/4 pot (pot-sized with the nuts), a
  3×BB open preflop, 3× 3-bets, looser thresholds in position and
  heads-up, and a push/fold mode when the stack drops to ~12 BB or less.
- **Safety:** every input is parsed defensively. Missing/empty fields,
  `min_bet = 0` (free check), all-in spots, and tiny turn times all degrade
  to a legal action — never a crash, never extra stdout. Logging goes to
  stderr only; stdout carries exactly one line per invocation.

## Assumptions about the API (not spelled out in `player_api.md`)

- `min_bet` is the **total** bet a player must have in the pot to stay in
  (`bets[]` are totals), so calling costs `min_bet - my_bet`.
- `RAISE <n>` is likewise a **total** bet target. Raises are always sized
  above `min_bet` and capped at the player's stack (all-in).
- `CALL` when `min_bet <= my_bet` is a free check.

## Test results

Local harness (`harness/harness.py`, full Texas Hold'em with blinds,
betting rounds, side pots, and showdown). All matches: 200 hands,
1000-chip stacks, 10/20 blinds, 1000 ms turn time. Net chips are for
PatBot; positive = PatBot wins. Full JSON in `harness/results/`.

| Opponent | Net chips/hand | Hands won | Showdowns won |
|---|---|---|---|
| python-example (always folds) | +15.0 | 200/200 (100%) | — (no showdowns) |
| Always-call bot | +85.5 | 97/200 (48.5%)* | 106/200 (53%) |
| Random bot | +69.1 | 162/200 (81%) | 20/35 (57%) |
| Tight premium-only bot | +23.9 | 153/200 (76.5%) | 27/59 (46%) |
| Itself (seed 22) | −55.5 | 87/200 (43.5%) | 58/131 (44%) |
| Itself (seed 99, rerun) | −9.3 | 91/200 (45.5%) | 73/133 (55%) |

\* vs always-call, "hands won" understates it: every hand goes to showdown
and PatBot wins the big pots and loses small ones.

- **Edge cases:** 21/21 pass (`harness/edge_tests.py`) — START prints only
  the name; `min_bet = 0`; all-in calls; short stacks; 3-player and
  multi-way pots; antes; 1 ms turn time; missing/empty/garbage fields.
  Zero invalid actions, timeouts, or crashes in any match.
- **Speed:** avg decision 48 ms unloaded (max 71 ms); ~100–160 ms under
  heavy parallel load — well inside a 1000 ms turn.
- **Evaluator:** exact on 20,000 random 7-card hands vs a brute-force
  reference (3 bugs found and fixed by that check: straight vs trips
  ordering, quads/two-pair kickers).
- **Self-play note:** the two self-matches average −32 chips/hand, but
  per-hand std is ~406 chips (32 all-in swings per 200 hands), so that is
  ~1.6 standard errors from zero — consistent with no edge and no seat
  bias (button/blinds verified symmetric). The bot's real edge shows in
  the +24 to +86 chips/hand against the non-identical baselines.
