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
  import { onMounted } from 'vue';
import { onUnmounted } from 'vue';
  import { Ref, ref } from 'vue';
  import { watch } from 'vue';

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
  const schedules = ref([]);
  onMounted(async () => {
    schedules.value = await (await fetch("/api/tracked_collection")).json();
    interval = setInterval(async () => {
      schedules.value = await (await fetch("/api/tracked_collection")).json();
    }, 2 * 1000);
  })

  onUnmounted(async () => {
    clearInterval(interval);
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
    schedules.value = await (await fetch("/api/schedule")).json();
    url.value = "";
  }

</script>
  