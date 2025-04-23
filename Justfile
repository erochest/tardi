
check:
    cargo check

test *FILTERS:
    cargo nextest run {{FILTERS}}

run TARDI_FILE *ARGS:
    cargo run -- --print-stack {{ARGS}} {{TARDI_FILE}}

repl *ARGS:
    cargo run -- --print-stack {{ARGS}}

build:
    cargo build

clean:
    cargo clean

watch:
    cargo watch -x "nextest run"

install:
    cargo install --path .

lint:
    cargo clippy --fix
    cargo fmt
    jj commit -m "cargo clippy fmt"
