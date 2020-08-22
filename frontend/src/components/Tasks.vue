<template>
    <v-card style="overflow-y: scroll; height: 85vh;" ref="card">
      <v-card-title>Tasks: {{tasks.length}}</v-card-title>
      <v-treeview
          :open="openedTasks(tasks)"
          :activatable="true"
          v-on:update:active="activeChanged"
          :return-object="true"
          :items="tasks"
          :item-children="'children'"
          :item-key="'id'"
          :item-text="'label'"
      ></v-treeview>
    </v-card>
</template>

<script lang="ts">
import Vue from "vue";
import Component from 'vue-class-component'
import {Task, TaskStatus} from "@/factorio-bot/task";
import {formatDuration} from "@/factorio-bot/util";


export default Vue.extend({
  name: "Tasks",
  props: {
    tasks: Array
  },
  methods: {
    openedTasks(tasks: Task[]): Task[] {
      const openTasks = tasks.filter(task => task.status !== TaskStatus.FINISHED)
      let retTasks = openTasks.map(task => task)
      for (let openTask of openTasks) {
        retTasks = retTasks.concat(this.openedTasks(openTask.children))
      }
      return retTasks
    },
    activeChanged(activeIds: Task[]): void {
      this.$emit('activeChanged', activeIds[0])
    }
  },
  watch: {
    tasks()  {
      const card: Element = (this.$refs.card as Vue).$el;
      card.scrollTo(0, card.scrollHeight)
    }
  }
});

</script>
