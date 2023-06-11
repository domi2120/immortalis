<template>
  <v-app>
    <v-main>
      <v-app-bar>
        <v-app-bar-nav-icon @click="drawerOpened= !drawerOpened" />
        <v-toolbar-title style="cursor: pointer" @click="clickNavigationEntryHome" >Immortalis</v-toolbar-title>
        <v-text-field :label="$t('search')" append-inner-icon="mdi-magnify" v-model="searchText" clearable class="w-50 mt-5 h-80" @keydown.enter="$router.push({ path: '/search', query: {searchText: searchText, t: Date.now()}})">
        </v-text-field>
        <v-spacer></v-spacer>
        <v-btn icon href="https://github.com/domi2120/immortalis">
          <v-icon>mdi-github</v-icon>
        </v-btn>
      </v-app-bar>
      <v-navigation-drawer v-model="drawerOpened" permanent>
        <v-list nav :mandatory="true">
          <v-list-item id="home" :value="'Home'" :title="$t('navigation.home')" router :to="`/?t${Date.now()}`" exact />
          <v-list-item :value="'Scheduling'" :title="$t('navigation.schedules')" router :to="'/scheduling'" exact />
          <v-list-item :value="'TrackedCollection'" :title="$t('navigation.trackedCollections')" router :to="'/tracked-collections'" exact/>
          
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
import { VListItem } from 'vuetify/components/VList';

// basically, not clicking the list and working just with the "to" prop, causes the old list entry to still be selected when changing the route by clicking an entry
// outside of the list (like the home button around the toolbar-title)
const clickNavigationEntryHome = () => (document.querySelector("#home") as HTMLAnchorElement).click();

const drawerOpened = ref(true);

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
  webSocket = new WebSocket(`${location.protocol === "https:" ? 'wss' : 'ws'}://${window.location.host}/api/ws/`)
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