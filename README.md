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

A Rust toolchain, then the three command line tools the project drives itself
with.

[`uv`](https://docs.astral.sh/uv/) manages the Python environment. Install the
prebuilt binary rather than building it from source, which is slow and needs a
recent rustc:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

[`just`](https://just.systems/) runs the project's commands and
[`maturin`](https://www.maturin.rs/) builds the Python extension:

```bash
cargo install just maturin
```

If `cargo install uv` is preferred, pin a version that your rustc supports;
cargo names one in the error when the latest is too new.

## Getting started

```bash
just --list
```

lists every command. The common ones:

```bash
just test          # Rust tests
just trial         # play random games and report win counts
just sync          # create or update the Python environment
```

`just sync` puts the virtualenv in `~/.venvs/dus-dus-dus` rather than in the
project, so it is not synced by any file syncing you have on the project
directory. Running `uv` directly instead of through `just` will create `.venv`
in the project unless you export `UV_PROJECT_ENVIRONMENT` yourself.

Build in release mode for anything that plays a lot of games. A debug build is
roughly sixteen times slower.

## Layout

| Path | What it is |
| --- | --- |
| `engine/` | The game: board, positions, rules, legal action enumeration, game records. No dependencies beyond serde. |
| `dus-dus-dus/` | Command line binary. Console and random players, and the game loop. |
| `game/` | An unused sketch of a game-agnostic framework with a Monte Carlo tree search. Nothing depends on it. |
| `scratch/` | A throwaway playground. |

## Status

The engine is complete and tested. There is no UI yet: `main()` runs a batch of
random games, and the console player exists but nothing currently reaches it.

Planned next are Python bindings for the engine, then a web UI with a Python
server, then multiplayer.
