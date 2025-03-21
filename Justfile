
check:
    cargo check

test *FILTERS:
    cargo nextest run {{FILTERS}}

build:
    cargo build

clean:
    cargo clean

watch:
    cargo watch -x "nextest run"
    
install:
    cargo install --path .

