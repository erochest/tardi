
test:
    cargo nextest run

build:
    cargo build

clean:
    cargo clean

watch:
    cargo watch -x "nextest run"
    
install:
    cargo install --path .
