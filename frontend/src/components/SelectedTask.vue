<template>
    <v-card v-if="task">
      <v-card-title>{{task.label}}</v-card-title>
      <v-list-item v-if="duration">{{ duration }}</v-list-item>
      <v-list-item v-if="task.result">{{ task.result }}</v-list-item>
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
  beforeUpdate() {
    if (this.$data.durationInterval === null && this.$props.task && this.$props.task.startedAt && !this.$props.task.finishedAt) {
      const updateDuration = () => {
        if (this.$props.task.finishedAt) {
          clearInterval(this.$data.durationInterval)
          this.$data.durationInterval = null
          this.$data.duration = null
          return
        }
        this.$data.duration = formatDuration(new Date().getTime() - this.$props.task.startedAt)
      }
      this.$data.durationInterval = setInterval(updateDuration)
      updateDuration()
    }
  },
  beforeDestroy() {
    if(this.$data.durationInterval) {
      clearInterval(this.$data.durationInterval);
      this.$data.durationInterval = null
    }
  },
  watch: {
    task()  {
      // console.log('TASK', this.$props.task)
    }
  },
  data() {
    return {
      duration: null,
      durationInterval: null,
    }
  }
});
</script>
