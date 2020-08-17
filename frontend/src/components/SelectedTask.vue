<template>
    <v-card  v-if="task">
      <v-card-title>Task #{{task.id}}: {{task.label}}</v-card-title>
      <v-list-item v-if="task.result">{{ task.result }}</v-list-item>


      <v-main>
        <v-list-item v-for="child in task.children" v-bind:key="child.id">Child #{{child.id}}: {{child.label}}</v-list-item>
      </v-main>
    </v-card>
</template>

<script lang="ts">
import Vue from "vue";
import {formatDuration} from "@/factorio-bot/util";

export default Vue.extend({
  name: "SelectedTask",
  props: {
    task: Object
  },
  methods: {
    formatDuration: formatDuration
  },
  watch: {
    task()  {
      console.log('TASK', this.$props.task)
    }
  }
});
</script>
