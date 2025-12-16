<script setup lang="ts">
	const canvasRef = ref<HTMLCanvasElement | null>(null);
	const { setMisterX, setDetective } = useGameCanvas(canvasRef);

	type ReplayData = {
		mister_x_starting_station: number;
		detective_starting_stations: { [key: string]: number };
		actions: (
			| {
					type: "mister_x";
					station: number;
					action_type: "taxi" | "bus" | "underground" | "water";
			  }
			| {
					type: "detective";
					color: string;
					station: number;
					action_type: "taxi" | "bus" | "underground";
			  }
		)[];
	};

	const replayData = ref<ReplayData | null>(null);

	watch(replayData, () => {
		setStartLocations();
	});

	function setStartLocations() {
		if (!replayData.value) return;

		setMisterX(replayData.value.mister_x_starting_station ?? 0);
		Object.entries(replayData.value.detective_starting_stations ?? {}).forEach(
			([color, station]) => {
				setDetective(color, station);
			},
		);
	}

	async function onPlay() {
		setStartLocations();

		for (const action of replayData.value?.actions ?? []) {
			if (action.type === "detective") {
				setDetective(action.color, action.station);
			} else if (action.type === "mister_x") {
				await new Promise((resolve) => setTimeout(resolve, 1000));
				setMisterX(action.station);
				await new Promise((resolve) => setTimeout(resolve, 1000));
			}
		}
	}

	const onFileChange = (event: Event) => {
		const file = (event.target as HTMLInputElement).files?.[0];
		if (!file) return;

		const reader = new FileReader();
		reader.onload = (e) => {
			const data = e.target?.result as string;
			try {
				replayData.value = JSON.parse(data);
			} catch (error) {
				console.error("Invalid JSON data");
			}
		};

		reader.readAsText(file);
	};
</script>

<template>
	<div class="h-dvh p-2 flex gap-2">
		<canvas ref="canvasRef" class="border w-full h-full"></canvas>
		<div class="w-48 h-full flex flex-col">
			<div>
				<h2 class="text-2xl">Replay</h2>
				<input
					class="cursor-pointer border w-full"
					type="file"
					@change="onFileChange"
				>
				<button class="border w-full cursor-pointer" @click="onPlay">
					Play
				</button>
			</div>
			<div class="flex flex-col gap-1 flex-1 overflow-scroll">
				<div v-for="action in replayData?.actions ?? []">
					<h3 class="text-xl">{{ action.type }}</h3>
					<div>
						{{ action.station }}
						{{ action.type }}
					</div>
				</div>
			</div>
		</div>
	</div>
</template>
