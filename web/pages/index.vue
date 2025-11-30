<script setup lang="ts">
import type { Connection, Station } from "~/utils/type/canvas";

const { data: stations } = await useFetch<Station[]>(
  "http://localhost:8081/map/stations",
);

const { data: connections } = await useFetch<Connection[]>(
  "http://localhost:8081/map/connections",
);

const canvasRef = ref<HTMLCanvasElement | null>(null);

const colors: Record<string, string> = {
  taxi: "yellow",
  bus: "green",
  underground: "red",
  water: "blue",
};

let zoom = 1;
let offsetX = 0;
let offsetY = 0;

let isDragging = false;
let dragStartX = 0;
let dragStartY = 0;

const STATION_DISTANCE = 20;

function draw() {
  if (!canvasRef.value || !stations || !connections) return;
  const ctx = canvasRef.value.getContext("2d");
  if (!ctx) return;

  ctx.save();
  ctx.fillStyle = "lightblue";
  ctx.fillRect(0, 0, canvasRef.value.width, canvasRef.value.height);

  ctx.translate(offsetX, offsetY);
  ctx.scale(zoom, zoom);

  connections.value!.forEach((conn) => {
    const fromStation = stations.value!.find((s) => s.id === conn.from);
    const toStation = stations.value!.find((s) => s.id === conn.to);
    if (!fromStation || !toStation) return;

    ctx.strokeStyle = colors[conn.mode]!;
    ctx.lineWidth = 4;
    ctx.beginPath();
    ctx.moveTo(fromStation.pos_x, fromStation.pos_y);
    ctx.lineTo(toStation.pos_x, toStation.pos_y);
    ctx.stroke();
  });

  stations.value!.forEach((station) => {
    ctx.fillStyle = "black";
    ctx.beginPath();
    ctx.arc(station.pos_x, station.pos_y, STATION_DISTANCE, 0, Math.PI * 2);
    ctx.fill();

    ctx.fillStyle = "white";
    ctx.font = "20px Arial";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(station.id.toString(), station.pos_x, station.pos_y);
  });

  ctx.restore();
}

onMounted(() => draw());
watch([stations, connections], () => draw());

function onWheel(event: WheelEvent) {
  const rect = canvasRef.value!.getBoundingClientRect();
  const mouseX = event.clientX - rect.left;
  const mouseY = event.clientY - rect.top;

  const scaleAmount = -event.deltaY * 0.001;
  const newZoom = Math.min(Math.max(zoom + scaleAmount, 0.1), 5);

  offsetX -= (mouseX - offsetX) * (newZoom / zoom - 1);
  offsetY -= (mouseY - offsetY) * (newZoom / zoom - 1);

  zoom = newZoom;
  draw();
}

function onMouseDown(event: MouseEvent) {
  if (event.button !== 1) return;
  isDragging = true;
  dragStartX = event.clientX - offsetX;
  dragStartY = event.clientY - offsetY;
}

function onMouseMove(event: MouseEvent) {
  if (!isDragging) return;
  offsetX = event.clientX - dragStartX;
  offsetY = event.clientY - dragStartY;
  draw();
}

function onMouseUp() {
  isDragging = false;
}

function onClick(event: MouseEvent) {
  if (!canvasRef.value || !stations.value) return;

  const rect = canvasRef.value.getBoundingClientRect();
  const mouseX = (event.clientX - rect.left - offsetX) / zoom;
  const mouseY = (event.clientY - rect.top - offsetY) / zoom;

  stations.value.forEach((station) => {
    const dx = mouseX - station.pos_x;
    const dy = mouseY - station.pos_y;
    const distance = Math.sqrt(dx * dx + dy * dy);
    if (distance <= STATION_DISTANCE) {
      onStationClick(station.id);
    }
  });
}

function onStationClick(id: number) {
  console.log("Station geklickt:", id);
}

const { data, send } = useGameConnection();

watch(data, (data) => {
  console.log("new message:", data);
});
send("createGame", { number_of_detectives: 4 });
</script>

<template>
  <div class="m-10">
    <canvas
      ref="canvasRef"
      width="1800"
      height="1000"
      class="border"
      @wheel.prevent="onWheel"
      @mousedown.prevent="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
      @click="onClick"
    ></canvas>
  </div>
</template>
