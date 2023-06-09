<template>
    <v-container class="ma-10">
      <div class="text-center text-h4 d-flex justify-center"> {{ $t('navigation.schedules') }} </div>
      
      <v-spacer></v-spacer>
      <v-col :cols="6" sm=12 class="d-flex flex-column mt-15 mb-15">
        <v-text-field :label="$t('scheduleView.address')" class="mt-10 w-50 d-flex flex-column align-self-center" v-model="url" ></v-text-field>
        <v-btn class="w-50 d-flex flex-column align-self-center" @click="schedule">{{ $t('scheduleView.schedule') }}</v-btn>
      </v-col>

      <v-data-table
      :headers="headers"
      :items="schedules"
      :items-per-page="5"
      class="elevation-1"
      >
        <template v-slot:[`item.scheduledAt`]="{ item }">
          {{ new Date(item.raw.scheduledAt).toLocaleDateString() }} {{ new Date(item.raw.scheduledAt).toLocaleTimeString() }}
        </template>

        <template v-slot:[`item.notBefore`]="{ item }">
          {{ new Date(item.raw.notBefore).toLocaleDateString() }} {{ new Date(item.raw.notBefore).toLocaleTimeString() }}
        </template>
      </v-data-table>
    </v-container>
  </template>
  
<script lang="ts" setup>
import { onMounted } from 'vue';
import { onUnmounted } from 'vue';
import { Ref, ref } from 'vue';
import { ScheduledArchival } from '@/models/scheduledArchival'
import Notyf from '@/notification';
import { WebSocketEvent } from '@/models/webSocketEvent';
import { DataChangeEvent } from '@/models/dataChangeEvent';
import { emitter } from '@/eventService';
import { useI18n } from 'vue-i18n';

const i18n = useI18n();

const url: Ref<string> = ref("");

const headers = ref(
  [
    {
      title: i18n.t('scheduleView.address'),
      value: 'url',
      align: 'start',
      //sortable: 'true'
    },
    {
      title: i18n.t('scheduleView.scheduledAt'),
      value: 'scheduledAt',
      key: 'scheduledAt',
      align: 'start'
    },
    {
      title: i18n.t('scheduleView.waitingUntil'),
      value: 'notBefore',
      key: 'notBefore',
      align: 'start'
    }
  ]
);

const schedules: Ref<ScheduledArchival[]> = ref([]);
onMounted(async () => {
  emitter.on("webSocketScheduledArchival", onWebSocketScheduledArchival);
  try {
    schedules.value = await (await fetch("/api/schedule")).json();
  } catch (e) {
    new Notyf().error(i18n.t("error.serverNotAvailable"));
  }
})

onUnmounted(async () => {
  emitter.off("webSocketScheduledArchival", onWebSocketScheduledArchival);
})

async function onWebSocketScheduledArchival(webSocketEvent: WebSocketEvent<DataChangeEvent<ScheduledArchival>>) {
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
  
async function schedule() {
  let response = await fetch("/api/schedule",
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
  response.ok ? new Notyf().success(i18n.t('scheduleView.success.scheduled', [url.value])) : new Notyf().error(i18n.t(`scheduleView.error.alreadyScheduled`, [url.value]))
  url.value = "";
}
</script>
  