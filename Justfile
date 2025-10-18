TARDI_NEXT := "cargo run --bin tardi-next --"

run *args:
  {{TARDI_NEXT}} {{args}}
