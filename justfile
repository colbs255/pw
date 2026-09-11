default: build

build:
    cargo build

run *ARGS:
    cargo run -- {{ARGS}}

test:
    cargo test

fmt:
    cargo fmt

lint:
    cargo clippy --all-targets --all-features -- -D warnings

check: fmt lint test

clean:
    cargo clean
