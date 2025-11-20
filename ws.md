# WebSocket Protocol Documentation

## Legend

colors: 'red', 'blue', 'green', 'yellow', 'purple'

## Connection Lifecycle

### Error
**Server → Client**\
[error] {message: string}

---

### Create a Game
**Client → Server**\
[createGame] { number_of_detectives: number }

**Server → Client**\
[game] {id: string}

---

### Join a Game
**Client → Server**\
[joinGame] { id: string }

---

## Gameplay Phase

### Start Game
**Client → Server**\
[startGame]

**Server → Client**\
[gameStarted] {role: 'detective' | 'mister_x'}

---

### Move Cycle

#### Begin Move
**Server → Clients**\
[startMove] {role: 'detective' | 'mister_x'}

#### Game State Updates (may repeat)
**Server → Detectives**\
[game] { players: [{ color: color, station_id: number, available_transport: {taxi: number, bus: number, underground: number} }, mister_x: {station_id: number | undefined, abilities: {hidden: number, double: number} }] }

**Server → MisterX**\
[game] { players: [{ color: color, station_id: number, available_transport: {taxi: number, bus: number, underground: number} }, mister_x: {station_id: number, abilities: {hidden: number, double: number} }] }

#### Player Move
**Detective → Server**\
[moveDetective] { detective_color: string, station_id: number, transport_type: 'taxi' | 'bus' | 'underground' }

**MisterX → Server**\
[moveMisterX] { station_id: number, transport_type: 'taxi' | 'bus' | 'underground' | 'hidden' }[]

#### Submit Move
**Client → Server**\
[submitMove]

#### Finish Move
**Server → Client**\
[endMove]

---

### Game End
**Server → Client**\
[gameEnded] {winner: 'detective' | 'mister_x'}
