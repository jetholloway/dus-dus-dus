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

A Rust toolchain, plus two tools:

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
just trial         # play random games and report win counts
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

## Playing in the browser

```bash
just serve
```

then open <http://127.0.0.1:8000>. Play against a random bot, or hot-seat
with two people on one browser. A game's address includes its id, so
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
| `dus-dus-dus/` | Command line binary. Console and random players, and the game loop. |
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
```

The browser UI plays full games against a random bot or hot-seat.

Saved games can be listed and replayed, and two people can play online
with an invite link.
