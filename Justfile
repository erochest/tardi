set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

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
    # TODO: clean out the `std/` directory before installing
    cargo install --path .
    -mkdir -p "{{clean(join(data_directory(), 'tardi', 'std'))}}"
    -cp -r std "{{clean(join(data_directory(), 'tardi', 'std'))}}"

update:
    jj git fetch
    jj new main
    just install

lint:
    cargo clippy --fix
    cargo fmt
    jj commit -m "cargo clippy fmt"

tasks:
    # TODO: this doesn't play nicely with pwsh
    rg --ignore-case "\\bxxx\\b|\\btodo\\b" src tests
