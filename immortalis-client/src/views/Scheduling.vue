<template>
    <v-container class="ma-10">
      <div class="text-center text-h4 d-flex justify-center"> Scheduling </div>
      
      <v-spacer></v-spacer>
      <v-col :cols="6" sm=12 class="d-flex flex-column mt-15 mb-15">
        <v-text-field :label="'Url'" class="mt-10 w-50 d-flex flex-column align-self-center" v-model="url" ></v-text-field>
        <v-btn class="w-50 d-flex flex-column align-self-center" @click="schedule">Schedule</v-btn>
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
  import { onMounted } from 'vue';
  import { onUnmounted } from 'vue';
  import { Ref, ref } from 'vue';
  import { ScheduledArchival } from '@/models/scheduledArchival'
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
        title: 'ScheduledAt',
        value: 'scheduledAt',
        align: 'start'
      },
      {
        title: 'Waiting untill',
        value: 'notBefore',
        align: 'start'
      }
    ]
  );

  let webSocket: WebSocket;
  const schedules: Ref<ScheduledArchival[]> = ref([]);
  onMounted(async () => {
    webSocket = new WebSocket(`ws://${window.location.host}/api/ws/`)
    webSocket.onmessage = async (x) => {
      let data: { record: ScheduledArchival, action: "delete" | "insert" } = JSON.parse(x.data);

      switch (data.action) {
        case "insert":
          //schedules.value = await (await fetch("/api/schedule")).json();
          schedules.value.push(data.record);
          break;
        case "delete":
          schedules.value.splice(schedules.value.findIndex(s => s.id == data.record.id))
          break;
      }
    }

    schedules.value = await (await fetch("/api/schedule")).json();
  })

  onUnmounted(async () => {
    webSocket.close();
  })
  
  async function schedule() {
    await fetch("/api/schedule",
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
    schedules.value = await (await fetch("/api/schedule")).json();
    url.value = "";
  }
</script>
  