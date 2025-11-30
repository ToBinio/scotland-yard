[working-directory: 'bots/bot_random']
run-random game_id:
  cargo run --release -- --server http://localhost:8081 --game-id {{game_id}}

[working-directory: 'bots/runtime']
create-game:
  cargo run --release -- create-game --server http://localhost:8081
