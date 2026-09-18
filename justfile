# Keep the virtualenv out of Dropbox: .gitignore does not stop Dropbox syncing.
export UV_PROJECT_ENVIRONMENT := env_var_or_default("UV_PROJECT_ENVIRONMENT", env_var("HOME") + "/.venvs/dus-dus-dus")

# List the available recipes.
default:
    @just --list

# Create or update the Python environment.
sync:
    uv sync

# Rebuild the Rust extension into the Python environment.
develop:
    uv run maturin develop --manifest-path bindings/Cargo.toml

# Run the Rust tests.
test:
    cargo test --workspace

# Run the Python tests.
test-py:
    uv run pytest

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
