<template>
  <v-app>
    <v-main>
      <v-app-bar>
        <v-app-bar-nav-icon @click="drawerOpened= !drawerOpened" />
        <v-toolbar-title>Immortalis</v-toolbar-title>
        <v-text-field label="Search" v-model="searchText" clearable class="w-50 mt-5 h-80" @keydown.enter="search">
        </v-text-field>
        <v-spacer></v-spacer>
      </v-app-bar>
      <v-navigation-drawer v-model="drawerOpened" permanent>
        <!-- 
        <v-list>
          <v-list-item 
            v-for="video in videos"
            :key="video.title"
            :title="video.title">
          </v-list-item>
        </v-list>
      -->
      </v-navigation-drawer>
      <v-container class="ma-10">
        <v-row v-for="video in videos" >
        <video-entry :video=ref(video) />
        </v-row>
      </v-container>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
  import { Ref, ref } from 'vue';
  import { Video } from '@/models/video';
  import VideoEntry from './components/VideoEntry.vue';

  const drawerOpened = ref(false);
  let videos: Ref<Video[]> = ref([]);

  let searchText: Ref<string> = ref("");

  const search = async () => {
    videos.value = await (await fetch("api/search?" + new URLSearchParams({term: `${searchText.value}`}))).json();
    videos.value.forEach((x: Video) => x.selectedDownload = x.downloads[0])
  }
  
  search();
</script>
<style>
</style>