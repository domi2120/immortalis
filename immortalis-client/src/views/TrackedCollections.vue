<template>
    <v-container class="ma-10">
      <div class="text-center text-h4 d-flex justify-center"> Tracked collections </div>
    
      <v-spacer></v-spacer>
      <v-col :cols="6" sm=12 class="d-flex flex-column mt-15 mb-15">
        <v-text-field :label="'Url'" class="mt-10 w-50 d-flex flex-column align-self-center" v-model="url" ></v-text-field>
        <v-btn class="w-50 d-flex flex-column align-self-center" @click="track">Start tracking</v-btn>
      </v-col>

      <v-data-table
      :headers="headers"
      :items="schedules"
      :items-per-page="5"
      class="elevation-1"
      >
        <template v-slot:[`item.trackingStartedAt`]="{ item }">
          {{ new Date(item.raw.trackingStartedAt).toLocaleDateString() }} {{ new Date(item.raw.trackingStartedAt).toLocaleTimeString() }}
        </template>

        <template v-slot:[`item.lastChecked`]="{ item }">
          {{ new Date(item.raw.lastChecked).toLocaleDateString() }} {{ new Date(item.raw.lastChecked).toLocaleTimeString() }}
        </template>
      </v-data-table>
    </v-container>
  </template>
  
<script lang="ts" setup>
import { TrackedCollection } from '@/models/trackedCollection';
import { WebSocketEvent } from '@/models/webSocketEvent';
import { DataChangeEvent } from '@/models/dataChangeEvent';
import { onMounted } from 'vue';
import { onUnmounted } from 'vue';
import { Ref, ref } from 'vue';
import { emitter } from '@/eventService';
import { Notyf } from 'notyf';

const url: Ref<string> = ref("");
  
const headers = ref(
  [
    {
      title: 'Url',
      value: 'url', // name of the property from which the value is drawn
      key: 'url', // key of the column, essential for custom slots
      align: 'start',
      //sortable: 'true'
    },
    {
      title: 'Started tracking at',
      value: 'trackingStartedAt',
      key: 'trackingStartedAt',
      align: 'start'
    },
    {
      title: 'Last Checked at',
      value: 'lastChecked',
      key: 'lastChecked',
      align: 'start'
    }
  ]
);

const schedules: Ref<TrackedCollection[]> = ref([]);

onMounted(async () => {
  emitter.on("webSocketTrackedCollection", onWebSocketTrackedCollection);
  try {
    schedules.value = await (await fetch("/api/tracked_collection")).json();
  } catch (e) {
    new Notyf().error("Could not reach Server");
  }
})

async function onWebSocketTrackedCollection (webSocketEvent: WebSocketEvent<DataChangeEvent<TrackedCollection>>) {
  switch (webSocketEvent.data.action) {
  case "insert":
    schedules.value.push(webSocketEvent.data.record);
    break;
  case "update":
    schedules.value.splice(schedules.value.findIndex(s => s.id == webSocketEvent.data.record.id), 1, webSocketEvent.data.record)
    break;
  case "delete":
    schedules.value.splice(schedules.value.findIndex(s => s.id == webSocketEvent.data.record.id), 1)
    break;
  }
}

onUnmounted(async () => {
  emitter.off("webSocketTrackedCollection", onWebSocketTrackedCollection);
})
  
async function track() {
  await fetch("/api/tracked_collection",
    {
      method: "POST",
      body: JSON.stringify(
        {
          url: url.value
        }),
      headers: {
        "Content-Type": "application/json",
      }
    }
  );
  url.value = "";
}

</script>
  