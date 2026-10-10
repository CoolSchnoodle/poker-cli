# Engine API Specification

Engines will be triggered
- At the start of each game
- Whenever they need to take a turn

Before triggering an engine for the first time, `engine-dir/build.sh` (where `engine-dir` is the directory with the engine's files) will be run, and at each trigger `enginer-dir/run.sh` will be run. For both, they will be run from `engine-dir`.

### Game Start

The engine will be supplied with two arguments: `START`, and a path to a directory that the engine can use as temporary storage during the game. The engine should send to stdout its engine name and nothing else. The engine name may contain whitespace, but a newline indicates the end of the name.

### Action Needed

The following arguments will be supplied to the engine, in order:
- The minimum amount that the engine must bet if they do not wish to fold
- The time the engine has for the turn, in milliseconds
- The engine's remaining time bank, in milliseconds
- A path\* to a file with the game parameters
- A path\* to a file with the current game state
- A path\* to a file with a list of all game actions
- A path\* to a directory that the engine can use as temporary storage during a single game

\*Each of these paths are guaranteed to be unquoted, whitespace-free, and valid UTF-8.

Here is an example of what the entire argument list might look like:

`0 1 2.345 game-params.txt game-state.txt game-actions.txt engine_tmp_storage/engine12345`

The engine should send exactly one of the following to stdout, and nothing else:
```
FOLD
CALL
RAISE <X>
```
Where `<X>` is a strictly positive whole number. If the engine bets more than it is able to, either through a call or a raise, it will go all in.

This output should be recieved in no more than the amount of time that the engine was alloted for the turn. If it is recieved later, the engine will begin to use some of their time bank. If the engine runs out of both their turn time and their time bank, they automatically fold.


## Treatment of stderr

Anything the engine outputs via stderr will be saved to a log file for the engine's developer to be able to consult later.


## Game Parameter File Format

The game parameter file format is a JSON file with the following format, although attributes may be in a different order and additional attributes may be added later:
```json
{
    "engines": [<engine names>],
    "small_blind": <integer>,
    "big_blind": <integer>,
    "ante": <integer>,
    "starting_funds": <integer>,
    "starting_time_bank": <integer>
}
```
Note: time values are always in milliseconds.


## Game State File Format

The game state file format is a JSON file with the following format, although attributes may be in a different order and additional attributes may be added later:
```json
{
    "you_index": <number>,
    "funds": [<unbet funds>],
    "bets": [<current bets>],
    "folds": [<current fold states>],
    "time_banks": [<time banks>],
    "your_hand": [<two cards>],
    "revealed_cards": [<revealed cards>],
    "small_blind": <small blind index>,
    "big_blind": <big blind index>
}
```
For example,
```json
{
    "you_index": 0,
    "funds": [380, 350, 400],
    "bets": [20, 50, 0],
    "folds": [false, false, true],
    "time banks": [9870, 6543, 10000],
    "your hand": [[14, 0], [7, 1]],
    "revealed cards": [],
    "small blind": 0,
    "big blind": 1
}
```
Note: time values are always in milliseconds.

Note: the indices for each engine are stable within a game.

Cards are an array with two elements, first the rank and then the suit. 2-10 represent the number cards, 11 is a Jack, 12 is a Queen, 13 is a King, and 14 is an Ace. Although the suit identities do not matter, we say that suit 0 represents Spades, 1 represents Clubs, 2 represents Hearts, and 3 represents Diamonds.


## Game Action List File Format

The game action list file is currently empty.

