<template>
  <v-container class="ma-10">
    <v-row v-for="video in videos" v-bind:key="video.id">
      <video-entry :video=ref(video) />
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import { Ref, ref } from 'vue';
import { Video } from '@/models/video';
import VideoEntry from '@/components/VideoEntry.vue';

let videos: Ref<Video[]> = ref([]);

const search = async () => {    
  videos.value = await (await fetch("api/search?" + new URLSearchParams({term: ""}))).json();
}

search();

</script>
