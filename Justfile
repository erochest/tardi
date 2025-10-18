TARDI_NEXT := "cargo run --bin tardi-next --"

test:
  cargo nextest run

run *args:
  {{TARDI_NEXT}} {{args}}

scan *args:
  {{TARDI_NEXT}} scan {{args}}

