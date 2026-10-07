env_name := "dus-dus-dus"

# `micromamba shell init` installs a shell function, which a recipe's
# non-interactive shell does not see. It does export MAMBA_EXE, so prefer that
# and fall back to a micromamba on PATH.
mamba := env_var_or_default("MAMBA_EXE", "micromamba")

# List the available recipes.
default:
    @just --list

# Create the Python environment, or update it if it already exists.
sync:
    #!/usr/bin/env bash
    set -euo pipefail
    if {{mamba}} env list | awk '{print $1}' | grep -qx '{{env_name}}'; then
        {{mamba}} env update -y -f environment.yml
    else
        {{mamba}} create -y -f environment.yml
    fi

# Rebuild the Rust extension into the Python environment.
develop:
    {{mamba}} run -n {{env_name}} maturin develop --manifest-path bindings/Cargo.toml

# Serve the game at http://127.0.0.1:8000, reloading when the code changes.
serve:
    {{mamba}} run -n {{env_name}} uvicorn --factory server.app:create_app --reload

# Run the Rust tests.
test:
    cargo test --workspace

# Run the Python tests.
test-py:
    {{mamba}} run -n {{env_name}} pytest

# Run every test.
test-all: test test-py

# Pit two bots against each other over N trials of two games each, e.g.
# `just arena heuristic random 500 --opening random:4`.
arena a="random" b="random" trials="500" *options="":
    cargo run --release -q -p dus-dus-dus -- arena {{a}} {{b}} --trials {{trials}} {{options}}

# Run a battery of arena matches, e.g. `just experiment heuristic-strength-grid
# --trials 200`. With no name, list them.
experiment *args="":
    {{mamba}} run -n {{env_name}} python experiments/run.py {{args}}

# Write saved games' openings to experiments/openings.txt, for --openings.
export-openings *args="":
    {{mamba}} run -n {{env_name}} python experiments/export_openings.py {{args}}

# Play in the terminal, against a bot or another person.
play:
    cargo run --release -q -p dus-dus-dus -- play

# Format and lint the Rust code.
fmt:
    cargo fmt --all

lint:
    cargo clippy --workspace --all-targets
