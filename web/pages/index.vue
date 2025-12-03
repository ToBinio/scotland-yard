<script setup lang="ts">
	const { data, send } = useGameConnection();

	watch(data, (message) => {
		if (!message) return;

		switch (message.type) {
			case "error":
				console.error(message.data.message);
				break;
			case "game":
				console.log("created game", message.data.id);
				break;
		}
	});
	send({ type: "createGame", data: { number_of_detectives: 4 } });
</script>

<template>
	<div class="p-10 h-dvh">
		<GameCanvas/>
	</div>
</template>
