<script setup lang="ts">
	const canvasRef = ref<HTMLCanvasElement | null>(null);
	const { setMisterX, setDetective } = useGameCanvas(canvasRef);
	setMisterX(1);
	setDetective("red", 2);
</script>

<template>
	<canvas ref="canvasRef" class="border w-full h-full"></canvas>
</template>
