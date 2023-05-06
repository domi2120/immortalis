<template>
    <v-container class="ma-10 ">
      <v-col :cols="1" sm=12 class="pa-3 w-100">
        
        <div class="text-center text-h4"> Scheduling </div>
        <v-text-field :label="'Url'" class="w-50 mt-10" v-model="url" ></v-text-field>
        <v-btn @click="schedule">Schedule</v-btn>
      </v-col>
    </v-container>
  </template>
  
  <script lang="ts" setup>
    import { Ref, ref } from 'vue';
    import { watch } from 'vue';

    const url: Ref<string> = ref("");
    
    async function schedule() {
      fetch("/api/schedule",
      {
        method: "POST",
        body: JSON.stringify(
          {
            url: url.value
          }),
          headers: {
            "Content-Type": "application/json",
            // 'Content-Type': 'application/x-www-form-urlencoded',
          }
        }
      );
    }
  </script>
  