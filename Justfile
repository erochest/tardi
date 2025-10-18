TARDI_NEXT := "cargo run --bin tardi-next --"

run *args:
  {{TARDI_NEXT}} {{args}}

scan *args:
  {{TARDI_NEXT}} scan {{args}}

