<script setup lang="ts">
const canvasRef = ref<HTMLCanvasElement | null>(null);
useGameCanvas(canvasRef);
</script>

<template>
  <canvas ref="canvasRef" class="border w-full h-full"></canvas>
</template>
