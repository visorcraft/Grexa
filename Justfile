set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Format every crate.
fmt:
    cargo fmt --all

# Cargo build (debug) for the entire workspace.
build:
    cargo build --workspace

# Cargo build (release) for the entire workspace.
build-release:
    cargo build --workspace --release

# Run all unit + integration tests.
test:
    cargo test --workspace

# Type-check without producing binaries.
check:
    cargo check --workspace

# Strict lint pass — the same gate CI uses.
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# `cargo-deny` license + advisory check (requires `cargo install cargo-deny`).
deny:
    cargo deny --all-features check

# Recursive `cargo-audit` against the lockfile (requires `cargo install cargo-audit`).
audit:
    cargo audit

# Generate and stamp the man page into target/man/.
manpage:
    mkdir -p target/man
    cargo run -q -p grexa-cli -- manpage > target/man/grexa-cli.1
    @echo "wrote target/man/grexa-cli.1"

# Generate shell completion scripts into target/completions/.
completions:
    mkdir -p target/completions
    cargo run -q -p grexa-cli -- completions bash > target/completions/grexa-cli.bash
    cargo run -q -p grexa-cli -- completions zsh > target/completions/_grexa-cli
    cargo run -q -p grexa-cli -- completions fish > target/completions/grexa-cli.fish
    @echo "wrote target/completions/*"

# `cargo package` for every crate (handy as a release dry-run).
package:
    cargo package --workspace --allow-dirty

# Run the CLI with arbitrary arguments. Example: `just run-cli /tmp TODO --quiet`.
run-cli *args:
    cargo run -p grexa-cli -- {{args}}

# Launch the GUI placeholder.
run-gui:
    cargo run -p grexa

# Convenience target — everything CI does. Useful before pushing.
ci: fmt lint test
    @echo "ci preflight passed"
