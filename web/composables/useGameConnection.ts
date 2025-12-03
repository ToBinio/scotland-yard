export type ClientMessage =
	| {
			type: "createGame";
			data: { number_of_detectives: number };
	  }
	| {
			type: "joinGame";
			data: { id: string };
	  }
	| {
			type: "startGame";
			data: null;
	  };

export type ServerMessage =
	| {
			type: "error";
			data: { message: string };
	  }
	| {
			type: "game";
			data: { id: string };
	  };

export function useGameConnection() {
	const { data: raw_data, send: raw_send } = useWebSocket<string>(
		"http://localhost:8081/game/ws",
	);

	const data = computed<ServerMessage | null>(() => {
		if (!raw_data.value) return null;

		const split = raw_data.value?.split(" ", 2);

		const message: ServerMessage = {
			type: split[0] ? split[0].substring(1, split[0].length - 1) : "?",
			data: split[1] ? JSON.parse(split[1]) : null,
		} as ServerMessage;

		return message;
	});

	function send(message: ClientMessage) {
		if (message.data) {
			raw_send(`[${message.type}] ${JSON.stringify(message.data)}`);
		} else {
			raw_send(`[${message.type}]`);
		}
	}

	return { data, send };
}
