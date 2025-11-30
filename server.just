[working-directory: 'server']
run:
  cargo run --release

[working-directory: 'server']
test:
  cargo nextest run
