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
    ></v-data-table>
    </v-container>
  </template>
  
<script lang="ts" setup>
  import { ScheduledArchival } from '@/models/scheduledArchival';
  import { TrackedCollection } from '@/models/trackedCollection';
  import { WebSocketEvent } from '@/models/webSocketEvent';
  import { DataChangeEvent } from '@/models/dataChangeEvent';
  import { onMounted } from 'vue';
  import { onUnmounted } from 'vue';
  import { Ref, ref } from 'vue';
  import { watch } from 'vue';
import consts from '@/consts';

  const url: Ref<string> = ref("");
  
  const headers = ref(
    [
      {
        title: 'Url',
        value: 'url',
        align: 'start',
        //sortable: 'true'
      },
      {
        title: 'Started tracking at',
        value: 'trackingStartedAt',
        align: 'start'
      },
      {
        title: 'Last Checked at',
        value: 'lastChecked',
        align: 'start'
      }
    ]
  );

  let interval: any;
  const schedules: Ref<TrackedCollection[]> = ref([]);
  let webSocket: WebSocket;

  onMounted(async () => {
    webSocket = new WebSocket(`ws://${window.location.host}/api/ws/`)
    webSocket.onmessage = async (x) => {
      let messsage: WebSocketEvent<DataChangeEvent<TrackedCollection>> = JSON.parse(x.data);
      switch (messsage.channel) {
        case consts.WebSocketChannels.TrackedCollections:
          switch (messsage.data.action) {
            case "insert":
              schedules.value.push(messsage.data.record);
              break;
            case "delete":
              schedules.value.splice(schedules.value.findIndex(s => s.id == messsage.data.record.id), 1)
              break;
          }
        break;
      }
    }

    schedules.value = await (await fetch("/api/tracked_collection")).json();
  })

  onUnmounted(async () => {
    webSocket.close();
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
  