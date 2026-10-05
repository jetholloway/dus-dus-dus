# dus-dus-dus

A board game engine in Rust, with Python bindings and a web UI.

## The game

The official rules are in [the rulebook](docs/DusDusDusv2booklet.pdf); this is
a summary.

Two players on a 7x7 board, files A to G and ranks 1 to 7. Player `O` starts
with seven pieces on rank 1, player `X` with seven on rank 7. There is one
ball. No piece is ever captured.

Each player opens by moving one piece exactly two squares; `O`'s opening move
also places the ball on the piece it moved. After that a turn is **three
actions**:

- **MOVE** one of your pieces one or two squares orthogonally, along a clear
  path, onto an empty square. The piece holding the ball may not move.
- **PASS** from the ball carrier to another of your pieces, in a straight line
  orthogonally or diagonally. Your own pieces do not block the lane; an
  opponent's piece does.
- **TACKLE** with a piece sharing an edge (not just a corner) with the
  opponent's piece that holds the ball. The tackler takes the ball and must
  then move one square in any direction, diagonals included, onto an empty
  square. It may not finish in the opponent's end zone.

Two kinds of stall are forbidden. Any action by either player that would
create one is illegal:

- **No wall stall.** A defender must leave a gap through which the opponent
  can bring new pieces into the defender's end zone. Only the defender's own
  pieces form a wall: the opponent's pieces never block the opponent, and one
  already in the end zone doesn't count as a way in. Pieces touching only at
  their corners still form a wall, because nothing moves diagonally.
- **No ball stall.** The player without the ball must be able to get a piece
  onto a square sharing an edge with the ball, or already have one there, so a
  tackle stays possible.

Neither applies during setup, while the back ranks are still full.

You win when the ball ends up on your opponent's back rank on one of your
pieces. Since the carrier cannot move and a tackle cannot land on a scoring
rank, that means passing the ball to a piece you already have waiting there.

Actions are written as `MOVE A1 A3`, `PASS A3 D3`, `TACKLE B4 B5`.

## Prerequisites

A Rust toolchain with `rustfmt` and `clippy`, plus two tools. With rustup,
`rust-toolchain.toml` installs the Rust components for you. With Debian's
Rust packages, install them yourself:

```bash
sudo apt install rustfmt rust-clippy
```

The two tools:

- [`micromamba`](https://mamba.readthedocs.io/) manages the Python environment.
  It also provides Python itself, so no system Python is needed.
- [`just`](https://just.systems/) runs the project's commands. On Debian:

```bash
sudo apt install just
```

`micromamba shell init` installs a shell function rather than a binary on
`PATH`, and recipes run in a non-interactive shell that does not see it. The
recipes therefore call `$MAMBA_EXE`, which that same setup exports, falling
back to a `micromamba` on `PATH`. If `just sync` reports that micromamba
cannot be found, check that `MAMBA_EXE` is exported in your shell.

`just` has to live outside the Python environment, because it is what creates
that environment. Everything else the project needs, including
[`maturin`](https://www.maturin.rs/), is listed in `environment.yml` and
installed by `just sync`.

## Getting started

```bash
just --list
```

lists every command. The common ones:

```bash
just sync          # create or update the Python environment
just develop       # build the Python bindings into the environment
just test          # Rust tests
just test-py       # Python tests, after just develop
just arena         # pit two bots against each other
just fmt           # format the Rust code
just lint          # check the Rust code with clippy
```

Re-run `just develop` after changing any Rust in `engine/` or `bindings/`;
Python only sees the engine as of the last build.

The environment is called `dus-dus-dus` and micromamba keeps it outside the
project directory, so nothing large lands in the repository. The recipes reach
into it with `micromamba run`, so they work whether or not it is activated.
Activating it also works if you would rather run `pytest` and `maturin`
directly:

```bash
micromamba activate dus-dus-dus
```

Build in release mode for anything that plays a lot of games. A debug build is
roughly sixteen times slower.

## Bots

Computer players live in the `bots` crate. Each implements a `Bot` trait
that picks one action at a time. There are two:

- `random` plays a uniformly random legal action. It is the baseline.
- `heuristic` tries every sequence of actions that finishes its turn,
  scores the position each one leads to, and plays the best. It does not
  look at the opponent's reply. A win scores +infinity; otherwise the score
  is a weighted sum of features, each counting for the bot and against it
  when the opponent has the same thing:

  | Weight | Default | Counts |
  | --- | --- | --- |
  | `possession` | 10 | holding the ball |
  | `ball_advance` | 2 | per rank the ball has advanced |
  | `piece_advance` | 0.5 | per rank each piece has advanced |
  | `receiver` | 3 | per piece on the opponent's back rank |
  | `open_lane` | 20 | the ball has a clear pass to one of those pieces |
  | `carrier_adjacent` | -6 | per opponent piece next to the ball holder |
  | `carrier_nearby` | -2 | per opponent piece 2 or 3 squares from it |

  Override any of them after a colon to try new values without
  recompiling, e.g. `heuristic:possession=12,open_lane=30`.

  `heuristic:pieces=N` weakens it: each turn it notices only N of its 7
  pieces at random and plans with those alone, so it can miss a win that
  needs a piece it overlooked. If none of them can act it notices one more.
  Over 1000 games each, against the full bot (`pieces=7`):

  | `pieces` | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
  | --- | --- | --- | --- | --- | --- | --- | --- |
  | wins vs full bot | 0.2% | 0.3% | 5% | 12% | 26% | 38% | 49% |
  | wins vs `random` | 98% | 99% | 100% | 100% | 100% | 100% | 100% |

The arena plays two bots against each other and reports how often each
wins:

```bash
just arena random random 1000
just arena heuristic:open_lane=30 heuristic 1000
```

Bot A plays `O` in even-numbered games and `X` in odd ones, so neither
gains from moving first. A game that reaches 3,000 actions is a draw, and
so is one where a player has no legal action. The report gives each bot's
wins overall and by side, a win rate with its 95% margin of error, the mean
time a bot takes per action, and the mean game length. Extra options go
after the game count, e.g. `just arena random random 1000 --seed 7`.
Everything is derived from the seed (default 0), so the same command gives
the same results.

`just play` plays a game in the terminal. Each side is `console`, where
you type actions like `MOVE A1 A3`, or a bot's name.

## Playing in the browser

```bash
just serve
```

then open <http://127.0.0.1:8000>. Play against a bot, choosing the
heuristic or the random one beside "New game vs" and whether you play
Orange, Teal or a side at random (the default); as Teal, the bot has
already made Orange's opening move. Whenever you play Teal, here or
online, the board and its replay are turned round so your side is at the
bottom. Or play hot-seat with two people on one browser. A game's address includes its id, so
reloading the page or bookmarking it returns to the same game.

Set a name in the box at the top. Your browser remembers it, along with a
random id that identifies it, and both are recorded against the seats you
take, so the game list shows who played what. Only the browser holding a
seat can move that side. Nothing is verified: players are trusted to name
themselves honestly.

"New online game" gives you one side at random and a link for the other.
Send the link to a friend: the first person to open it takes the empty seat,
and after that the link does nothing. Until they join, neither side can move.

An open game page checks with the server every two seconds, while the tab is
visible and the game unfinished, so your opponent's moves and their joining
appear on their own. Anyone watching a game sees it update the same way.

The front page lists every saved game. Any of them can be replayed a move
at a time, with the arrow keys or the controls under the board, and a game
still in progress can be picked up where it was left.

A rule change can make a move in an older game illegal. Such a game is
marked in the list, and its replay runs up to that move and explains why it
stops there.

Games are saved in SQLite at `~/.local/share/dus-dus-dus/games.sqlite`, or
wherever `DUS_DB` points. It is kept out of the project directory on purpose:
a file-syncing service copying a live database mid-write can corrupt it.

## Layout

| Path | What it is |
| --- | --- |
| `engine/` | The game: board, positions, rules, legal action enumeration, game records. No dependencies beyond serde. |
| `bots/` | Computer players, behind a `Bot` trait. |
| `dus-dus-dus/` | Command line binary: the bot arena and terminal play. |
| `bindings/` | The `dus_engine` Python module, built with maturin. Outside the Cargo workspace so Rust builds never need Python. |
| `server/` | The web server: a FastAPI JSON API over the engine, SQLite storage, and the browser client in `server/static/`. |
| `tests/` | Python tests for the bindings and the server. |
| `game/` | An unused sketch of a game-agnostic framework with a Monte Carlo tree search. Nothing depends on it. |
| `scratch/` | A throwaway playground. |

## Status

The engine is complete and tested, and usable from Python:

```python
from dus_engine import Action, GameState

state = GameState()
state.valid_actions()                 # [Action('MOVE A1 A3'), ...]
state = state.apply(Action("MOVE A1 A3"))
state.winner                          # None until someone scores

from dus_engine import Bot

bot = Bot("heuristic", seed=1)        # any name the arena takes
bot.choose(state)                     # Action('MOVE B7 B5')
```

The browser UI plays full games against either bot or hot-seat.

Saved games can be listed and replayed, and two people can play online
with an invite link.
