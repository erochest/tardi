
check:
    cargo check --tests

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

init:
    mkdir tmp

install:
    cargo install --path .
    mkdir -p {{data_directory()}}
    cp -r std {{data_directory()}}/std

update:
    jj git fetch
    jj new main
    just install

lint:
    cargo clippy --fix
    cargo fmt
    jj commit -m "cargo clippy fmt"

tasks:
    rg --ignore-case "\\bxxx\\b|\\btodo\\b" src tests
