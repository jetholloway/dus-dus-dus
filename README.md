# dus-dus-dus

A board game engine in Rust, with Python bindings and a web UI planned.

## The game

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
- **TACKLE** with a piece standing orthogonally adjacent to the ball: it steps
  onto an adjacent empty square and takes the ball with it. Illegal if your own
  side already holds the ball, or if it would land on a scoring rank.

After every action the ball must still be able to reach both rank 1 and rank 7
across empty squares, so neither player can wall it in.

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

## Layout

| Path | What it is |
| --- | --- |
| `engine/` | The game: board, positions, rules, legal action enumeration, game records. No dependencies beyond serde. |
| `dus-dus-dus/` | Command line binary. Console and random players, and the game loop. |
| `bindings/` | The `dus_engine` Python module, built with maturin. Outside the Cargo workspace so Rust builds never need Python. |
| `tests/` | Python tests for the bindings. |
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

There is no UI yet: `main()` runs a batch of random games, and the console
player exists but nothing currently reaches it.

Planned next are a web UI with a Python server, then multiplayer.
