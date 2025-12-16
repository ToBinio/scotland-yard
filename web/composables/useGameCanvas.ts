import type { Connection, Station } from "~/utils/type/canvas";

const MODE_DATA: Record<
  "taxi" | "bus" | "underground" | "water",
  { color: string; width: number; index: number }
> = {
  taxi: { color: "yellow", width: 2, index: 2 },
  bus: { color: "green", width: 6, index: 1 },
  underground: { color: "red", width: 12, index: 0 },
  water: { color: "blue", width: 4, index: 3 },
};

const STATION_DISTANCE = 20;
const MIDDLE_MOUSE_BUTTON = 1;

export function useGameCanvas(canvasRef: Ref<HTMLCanvasElement | null>) {
  const { data: stations } = useFetch<Station[]>(
    "http://localhost:8081/map/stations",
  );

  const { data: connections } = useFetch<Connection[]>(
    "http://localhost:8081/map/connections",
  );

  const sorted_connections = computed(() => {
    if (!connections.value) return [];

    return connections.value.sort(
      (a, b) => MODE_DATA[a.mode].index - MODE_DATA[b.mode].index,
    );
  });

  let zoom = 1;
  let offsetX = 0;
  let offsetY = 0;

  let isDragging = false;
  let dragStartX = 0;
  let dragStartY = 0;

  function draw() {
    if (
      !canvasRef.value ||
      !stations.value ||
      !connections.value ||
      !sorted_connections.value
    )
      return;
    const ctx = canvasRef.value.getContext("2d");
    if (!ctx) return;

    ctx.save();
    ctx.fillStyle = "lightblue";
    ctx.fillRect(0, 0, canvasRef.value.width, canvasRef.value.height);

    ctx.translate(offsetX, offsetY);
    ctx.scale(zoom, zoom);

    for (const conn of sorted_connections.value!) {
      const fromStation = stations.value!.find((s) => s.id === conn.from);
      const toStation = stations.value!.find((s) => s.id === conn.to);
      if (!fromStation || !toStation) return;

      ctx.strokeStyle = MODE_DATA[conn.mode].color;
      ctx.lineWidth = MODE_DATA[conn.mode].width;
      ctx.beginPath();
      ctx.moveTo(fromStation.pos_x, fromStation.pos_y);
      ctx.lineTo(toStation.pos_x, toStation.pos_y);
      ctx.stroke();
    }

    for (const station of stations.value!) {
      ctx.fillStyle = "black";
      ctx.beginPath();
      ctx.arc(station.pos_x, station.pos_y, STATION_DISTANCE, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = "white";
      ctx.font = "20px Arial";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(station.id.toString(), station.pos_x, station.pos_y);
    }

    ctx.restore();
  }

  onMounted(() => {
    if (canvasRef.value) {
      const observer = new ResizeObserver(resizeCanvas);
      observer.observe(canvasRef.value);

      canvasRef.value.addEventListener("wheel", onWheel);
      canvasRef.value.addEventListener("mousedown", onMouseDown);
      canvasRef.value.addEventListener("mousemove", onMouseMove);
      canvasRef.value.addEventListener("mouseup", onMouseUp);
      canvasRef.value.addEventListener("click", onClick);
    }

    resizeCanvas();
  });
  watch([stations, connections], () => draw());

  function resizeCanvas() {
    if (!canvasRef.value) return;

    const rect = canvasRef.value!.getBoundingClientRect();
    canvasRef.value!.width = rect.width;
    canvasRef.value!.height = rect.height;

    draw();
  }

  function onWheel(event: WheelEvent) {
    event.preventDefault();

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
    event.preventDefault();

    if (event.button !== MIDDLE_MOUSE_BUTTON) return;

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

  function onMouseUp(event: MouseEvent) {
    if (event.button !== MIDDLE_MOUSE_BUTTON) return;

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
}
