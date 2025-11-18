<script setup lang="ts">
type Station = {
  id: number;
  pos_x: number;
  pos_y: number;
  types: ("taxi" | "bus" | "underground")[];
};

let { data: stations } = await useFetch<Station[]>(
  "http://localhost:8080/map/stations",
);
</script>

<template>
  <div>
    <h1 class="text-3xl font-bold">Scotland Yard</h1>
    <div v-if="stations">
      <h2 class="text-2xl font-bold">Stations</h2>
      <div>
        <div class="flex gap-4" v-for="station in stations">
          <div>{{ station.id }}</div>
          <div>{{ station.pos_x }}</div>
          <div>{{ station.pos_y }}</div>
          <div>{{ station.types.join(", ") }}</div>
        </div>
      </div>
    </div>
  </div>
</template>
