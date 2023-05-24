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
  import { Ref, ref, onMounted } from 'vue';
  import { Video } from '@/models/video';
  import router from './router';
  import { useRoute } from 'vue-router';

  const drawerOpened = ref(false);
  let videos: Ref<Video[]> = ref([]);

  let searchText: Ref<string> = ref("");

  const search = async () => {
    videos.value = await (await fetch("api/search?" + new URLSearchParams({term: `${searchText.value}`}))).json();
  }

  onMounted(async () => {
    await router.isReady();
    router.afterEach(() => searchText.value = router.currentRoute.value.query.searchText?.toString() || "");
    searchText.value = router.currentRoute.value.query.searchText?.toString() || "";
  })
  
  
  search();
</script>
<style>
</style>