<template>
    <v-container class="ma-10 ">
      <v-col :cols="1" sm=12 class="pa-3 w-100">
        
        <div class="text-center text-h4"> Scheduling </div>
        <v-text-field :label="'Url'" class="w-50 mt-10" v-model="url" ></v-text-field>
        <v-btn @click="schedule">Schedule</v-btn>
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

  
  const schedules = ref([]);
  onMounted(async () => {
    schedules.value = await (await fetch("/api/schedule")).json();
    setInterval(async () => {
      schedules.value = await (await fetch("/api/schedule")).json();
    }, 2 * 1000);
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
  