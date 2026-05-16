set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

fmt:
    cargo fmt --all

build:
    cargo build --workspace

test:
    cargo test --workspace

check:
    cargo check --workspace

lint:
    cargo clippy --workspace --all-targets -- -D warnings

package:
    cargo package --workspace --allow-dirty

run-cli *args:
    cargo run -p grexa-cli -- {{args}}

run-gui:
    cargo run -p grexa
