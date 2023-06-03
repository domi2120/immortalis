<template>
  <v-app>
    <v-main>
      <v-app-bar>
        <v-app-bar-nav-icon @click="drawerOpened= !drawerOpened" />
        <v-toolbar-title style="cursor: pointer" @click="$router.push({path: '/', query: { t: Date.now()}})">Immortalis</v-toolbar-title>
        <v-text-field label="Search" append-inner-icon="mdi-magnify" v-model="searchText" clearable class="w-50 mt-5 h-80" @keydown.enter="$router.push({ path: '/search', query: {searchText: searchText, t: Date.now()}})">
        </v-text-field>
        <v-spacer></v-spacer>
      </v-app-bar>
      <v-navigation-drawer v-model="drawerOpened" permanent>
        <v-list nav :mandatory="true">
          <v-list-item :value="'Home'" :title="'Home'" @click="$router.push({path: '/', query: { t: Date.now()}})"/>
          <v-list-item :value="'Scheduling'" :title="'Scheduling'" @click="$router.push('/scheduling')"/>
          <v-list-item :value="'TrackedCollection'" :title="'Tracked Collections'" @click="$router.push('/tracked-collections')"/>
        </v-list>
      </v-navigation-drawer>
      <router-view :key="$route.fullPath"></router-view>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import { Ref, ref, onMounted, onUnmounted } from 'vue';
import router from './router';
import { emitter } from '@/eventService';
import { WebSocketEvent } from './models/webSocketEvent';
import consts from './consts';

const drawerOpened = ref(false);

let searchText: Ref<string> = ref("");

let webSocket: WebSocket;
let webSocketReconnectInterval: any; // handle to the interval

onMounted(async () => {
  await router.isReady();
  router.afterEach(() => searchText.value = router.currentRoute.value.query.searchText?.toString() || "");
  searchText.value = router.currentRoute.value.query.searchText?.toString() || "";
    
  connectWebsocket();
  webSocketReconnectInterval = setInterval(() => {
    if (webSocket.readyState === webSocket.CLOSED) {
      connectWebsocket();
    }
  }, 5000)
})

async function connectWebsocket(){
  webSocket = new WebSocket(`ws://${window.location.host}/api/ws/`)
  webSocket.onmessage = async (x) => {
    let message: WebSocketEvent<any> = JSON.parse(x.data);
    switch (message.channel) {
    case consts.WebSocketChannels.ScheduledArchivals:
      emitter.emit("webSocketScheduledArchival", message);
      break;
    case consts.WebSocketChannels.TrackedCollections:
      emitter.emit("webSocketTrackedCollection", message);
      break;
    default:
      console.log(`[WARNING] received a message on unknown channel ${message.channel}`);
      break;
    }
  };
}
  
onUnmounted(async () => {
  clearInterval(webSocketReconnectInterval);
  webSocket.close();
})
</script>
<style>
</style>