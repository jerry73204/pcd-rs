default:
    @just --list

build:
    cargo build --profile dev-release --all-targets --all-features

format:
    cargo +nightly fmt

check:
    cargo +nightly fmt --check
    cargo clippy --profile dev-release --all-targets --all-features

test:
    cargo nextest run --cargo-profile dev-release --no-fail-fast --all-features

ci: check test
