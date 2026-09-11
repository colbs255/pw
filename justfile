default: build

build:
    cargo build

run *ARGS:
    cargo run -- {{ARGS}}

test:
    cargo test

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy --all-targets --all-features -- -D warnings

check: fmt-check lint test

clean:
    cargo clean
