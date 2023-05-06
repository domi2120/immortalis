<template>
  <v-container class="ma-10">
    <v-row v-for="video in videos" >
      <video-entry :video=ref(video) />
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
  import { Ref, ref } from 'vue';
  import { Video } from '@/models/video';
  import VideoEntry from '@/components/VideoEntry.vue';
  import { watch } from 'vue';

  let videos: Ref<Video[]> = ref([]);

  const props = defineProps<
      {
          searchText: string
      }
  >();
  
  const watchers = watch(() => props.searchText, () => search() )

  const search = async () => {    
    videos.value = await (await fetch("api/search?" + new URLSearchParams({term: props.searchText}))).json();
    videos.value.forEach((x: Video) => x.selectedDownload = x.downloads[0])
  }

  search();

</script>
