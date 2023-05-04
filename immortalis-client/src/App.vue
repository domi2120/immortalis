<template>
  <v-app>
    <v-main>
      <v-app-bar>
        <v-app-bar-nav-icon @click="drawerOpened= !drawerOpened" />
        <v-toolbar-title>Immortalis</v-toolbar-title>
        <v-text-field label="Search" clearable class="w-50 mt-5 h-80" @keydown.enter="search">
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
          <v-spacer></v-spacer>
          <v-col :cols="1" sm=2 class="pa-3 ma-1">
            <v-img :src="video.thumbnailAddress" class="d-flex align-end" >
              <v-chip class="d-float float-right ">
                {{ new Date(video.duration * 1000).toISOString().slice(11, 19) }}
              </v-chip>
            </v-img>
          </v-col>
          <v-col :cols="3">
            <h2 >{{ video.title }}</h2>
            {{ video.channel }} <br>
            {{ numberToDelimetedString(video.views, ",") }} views · uploaded: {{ video.uploadDate.toLocaleDateString() }} · archived: {{ video.archivedDate .toLocaleDateString() }} <br>
            <v-select label="Download" :items="video.downloads" v-model="video.selectedDownload" class="w-40" return-object/>
            <v-btn @click="download(video)">Download</v-btn>
            <v-btn :href="video.originalUrl" class="ma-2">Watch Original</v-btn>
          </v-col>
          <v-spacer></v-spacer>
        </v-row>
      </v-container>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
  import { Ref, ref } from 'vue';
  import { Video } from '@/models/video';

  const drawerOpened = ref(false);
  let videos: Ref<Video[]> = ref([]);

  const search = () => videos.value = [{
      title: "Ghost - Rats (Official Music Video)",
      channel: "Ghost",
      views: 5000000,
      uploadDate: new Date(),
      archivedDate: new Date(),
      duration: 265,
      thumbnailAddress: "https://img.youtube.com/vi/C_ijc7A5oAc/maxresdefault.jpg",
      originalUrl: "https://www.youtube.com/watch?v=C_ijc7A5oAc",
      downloads: [
        {
          title: "Download(1080p30)",
          value: "Download(1080p30)"
        },
        {
          title: "Audio Only",
          value: "Audio Only"
        }
      ],
      selectedDownload: 
        {
          title: "Download(1080p30)",
          value: "Download(1080p30)"
        }
  },
  {
      title: "I Am",
      channel: "Theocracy - Topic",
      views: 388000,
      uploadDate: new Date(),
      archivedDate: new Date(),
      duration: 660,
      thumbnailAddress: "https://img.youtube.com/vi/vfc8EjDuYNw/maxresdefault.jpg",
      originalUrl: "https://www.youtube.com/watch?v=vfc8EjDuYNw",
      downloads: [
        {
          title: "Download(1080p30)",
          value: "Download(1080p30)"
        },
        {
          title: "Audio Only",
          value: "Audio Only"
        }
      ],
      selectedDownload: 
        {
          title: "Download(1080p30)",
          value: "Download(1080p30)"
        }
  }];

  search();
  function numberToDelimetedString(x: number, delimeter: string) {
    return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, delimeter);
  }

  function download(video: any) {
    console.log(video.title + " " + video.selectedDownload.value);
  }
  
</script>
<style>
</style>