<template>
    <v-container class="fill-height" v-if="props.video.value.title" >
        <v-spacer></v-spacer>
        <v-col :cols="2" sm=2 class="pa-3">
            <v-img :src="'/api/file?is_thumbnail=true&file_id=' + encodeURI(props.video.value.thumbnailId)" class="d-flex align-end" >
              <v-chip class="d-float float-right " variant="elevated">
                {{ new Date(props.video.value.duration * 1000).toISOString().slice(11, 19) }}
              </v-chip>
            </v-img>
          </v-col>
          <v-col :cols="4">
            <h2 >{{ props.video.value.title }}</h2>
            {{ props.video.value.channel }} <br>
            {{ numberToDelimetedString(props.video.value.views, ",") }} views · uploaded: {{ new Date(props.video.value.uploadDate).toLocaleDateString() }} · archived: {{ new Date(props.video.value.archivedDate).toLocaleDateString() }} <br>
            <!-- temporarily removed the data for it doesn't exist yet -->
            <!-- 
            <v-select :disabled="props.video.value.downloads.length < 1" label="Download" :items="props.video.value.downloads" v-model="props.video.value.selectedDownload" class="w-40" return-object/>
            -->
            <v-btn :href="'/api/file?is_thumbnail=false&file_id=' + encodeURI(props.video.value.fileId )" :disabled="props.video.value.status != 'Archived'" @click="download(video)" >Download</v-btn>
            <v-btn :href="props.video.value.originalUrl" class="ma-2">Watch Original</v-btn>
            <v-chip>{{ props.video.value.status }}</v-chip>
          </v-col>
        <v-spacer></v-spacer>
    </v-container>
</template>
  
<script setup lang="ts">

import { Video } from '@/models/video';
import { ref } from 'vue';
import { Ref } from 'vue';

let video: Ref<Video | {}> = ref({});

const props = defineProps<
    {
        video: Ref<Video>
    }
>();

function numberToDelimetedString(x: number, delimeter: string) {
    return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, delimeter);
}

async function download(video: any) {
  // not currently in use
  //console.log(video.title + " " + video.selectedDownload.value);
}

</script>
  