[working-directory: 'bots/bot_random']
run-random game_id:
  cargo run --release -- --server http://localhost:8081 --game-id {{game_id}}

[working-directory: 'bots/bot_random']
build-random:
  cargo build --release

[working-directory: 'bots/runtime']
create-game:
  cargo run --release -- create-game --server http://localhost:8081

[working-directory: 'bots/runtime']
run-battle:
  cargo run -- run-game --server http://localhost:8081 --bot-a ../../target/release/bot_random --bot-b ../../target/release/bot_random
