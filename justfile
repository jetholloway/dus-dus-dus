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

# Run the Rust tests.
test:
    cargo test --workspace

# Run the Python tests.
test-py:
    {{mamba}} run -n {{env_name}} pytest

# Run every test.
test-all: test test-py

# Play random games and report the win counts. Slow without --release.
trial:
    cargo run --release -p dus-dus-dus

# Format and lint the Rust code.
fmt:
    cargo fmt --all

lint:
    cargo clippy --workspace --all-targets
