<script setup lang="ts">
	import {
		DialogContent,
		DialogOverlay,
		DialogPortal,
		DialogRoot,
		DialogTitle,
	} from "reka-ui";

	const { data, send } = useGameConnection();

	watch(data, (message) => {
		if (!message) return;

		switch (message.type) {
			case "error":
				console.error(message.data.message);
				break;
			case "game":
				console.log("created game", message.data.id);
				gameID.value = message.data.id;
				break;
		}
	});

	function onCreateGame() {
		send({ type: "createGame", data: { number_of_detectives: 4 } });
	}

	const gameID = ref<string | null>(null);
	watch(gameID, (id) => {
		if (!id) return;

		send({ type: "joinGame", data: { id } });
		createGameDialogOpen.value = false;
	});

	let selectedGameID = ref<string>("");
	function onJoinGame() {
		if (!selectedGameID.value) return;
		gameID.value = selectedGameID.value;
	}

	const createGameDialogOpen = ref(true);
</script>

<template>
	<div class="p-10 h-dvh">
		<div>{{ gameID }}</div>
		<GameCanvas />

		<DialogRoot :open="createGameDialogOpen">
			<DialogPortal>
				<DialogOverlay
					class="data-[state=open]:animate-overlayShow fixed inset-0 z-100 backdrop-blur-[2px]"
				/>
				<DialogContent
					class="data-[state=open]:animate-contentShow fixed top-[50%] left-[50%] max-h-[30vh] h-[90vh] w-[90vw] max-w-120 translate-x-[-50%] translate-y-[-50%] focus:outline-none z-100 bg-white p-2 rounded"
				>
					<DialogTitle class="text-2xl"> Create or Join Game </DialogTitle>
					<div class="flex flex-col gap-4 items-start pt-4">
						<button
							@click="onCreateGame"
							type="button"
							class="px-2 py-1 bg-gray-500 text-white rounded hover:bg-gray-600 cursor-pointer"
						>
							Create Game
						</button>

						<div class="flex flex-col gap-1 items-start">
							<input
								type="text"
								placeholder="Game ID"
								v-model="selectedGameID"
								class="px-2 py-1 bg-gray-50 text-gray-900 rounded border border-gray-600"
							>
							<button
								@click="onJoinGame"
								type="button"
								class="px-2 py-1 bg-gray-500 text-white rounded hover:bg-gray-600 cursor-pointer"
							>
								Join Game
							</button>
						</div>
					</div>
				</DialogContent>
			</DialogPortal>
		</DialogRoot>
	</div>
</template>
