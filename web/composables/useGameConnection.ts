export type Message = {
	name: string;
	body: any | null;
};

export function useGameConnection() {
	const { data: raw_data, send: raw_send } = useWebSocket<string>(
		"http://localhost:8081/game/ws",
	);

	const data = computed<Message | null>(() => {
		if (!raw_data.value) return null;

		const split = raw_data.value?.split(" ", 2);

		const message: Message = {
			name: split[0] ? split[0].substring(1, split[0].length - 1) : "?",
			body: split[1] ? JSON.parse(split[1]) : null,
		};

		return message;
	});

	function send(name: string, message: any) {
		if (message) {
			raw_send(`[${name}] ${JSON.stringify(message)}`);
		} else {
			raw_send(`[${name}]`);
		}
	}

	return { data, send };
}
