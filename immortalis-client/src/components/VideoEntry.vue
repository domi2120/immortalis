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
            {{ $n(props.video.value.views) }} {{ $t('videoEntryComponent.views')}} · {{ $t('videoEntryComponent.uploadedAt')}}: {{ $d(new Date(props.video.value.uploadDate)) }} · {{ $t('videoEntryComponent.archivedAt') }}: {{ $d(new Date(props.video.value.archivedDate)) }} <br>
            <v-btn :href="'/api/file?is_thumbnail=false&file_id=' + encodeURI(props.video.value.fileId )" :disabled="props.video.value.status != 'Archived'" >{{ $t('videoEntryComponent.download') }}</v-btn>
            <v-btn :href="props.video.value.originalUrl" class="ma-2">{{ $t('videoEntryComponent.watchOriginal') }}</v-btn>
            <v-chip>{{ $t('videoEntryComponent.status.' + props.video.value.status) }}</v-chip>
            <v-chip>{{ prettyBytes(props.video.value.videoSize) }}</v-chip>
          </v-col>
        <v-spacer></v-spacer>
    </v-container>
</template>
  
<script setup lang="ts">

import { Video } from '@/models/video';
import { Ref } from 'vue';
import prettyBytes from 'pretty-bytes';

const props = defineProps<
  {
      video: Ref<Video>
  }
>();
</script>
  