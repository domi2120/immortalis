<template>
  <v-container class="ma-10">
    <v-row v-if="videos.length < 1">
      <v-spacer/>
      <v-col class="ma-10">{{ $t("noResultsFound") }}</v-col>
    </v-row>
    <v-row v-for="video in videos" v-bind:key="video.id" >
      <video-entry :video=ref(video) />
    </v-row>
  </v-container>
</template>
  
<script lang="ts" setup>
import { Ref, ref } from 'vue';
import { Video } from '@/models/video';
import VideoEntry from '@/components/VideoEntry.vue';
import { notyfInstance } from '@/notification';
import { useI18n } from 'vue-i18n';

const i18n = useI18n();
  
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
    notyfInstance.error(i18n.t("error.serverNotAvailable"));
  }
}
  
search();
  
</script>
  