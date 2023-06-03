<template>
    <v-container class="ma-10">
      <v-row v-for="video in videos" v-bind:key="video.id" >
        <video-entry :video=ref(video) />
      </v-row>
    </v-container>
  </template>
  
<script lang="ts" setup>
import { Ref, ref } from 'vue';
import { Video } from '@/models/video';
import VideoEntry from '@/components/VideoEntry.vue';
import { Notyf } from 'notyf';
  
let videos: Ref<Video[]> = ref([]);
  
const props = defineProps<
        {
            searchText: string
        }
    >();
      
const search = async () => {
  try {
    videos.value = await (await fetch("api/search?" + new URLSearchParams({term: props.searchText}))).json();
  } catch (e) {
    new Notyf().error("Could not reach Server");
  }
}
  
search();
  
</script>
  