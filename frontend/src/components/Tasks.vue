<template>
    <v-card>
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

// Define the props by using Vue's canonical way.
const TasksProps = Vue.extend({
  props: {
    tasks: Array
  }
})

@Component
export default class Tasks extends TasksProps {
  openedTasks(tasks: Task[]): Task[] {
    const openTasks = tasks.filter(task => task.status !== TaskStatus.FINISHED)
    let retTasks = openTasks.map(task => task)
    for (let openTask of openTasks) {
      retTasks = retTasks.concat(this.openedTasks(openTask.children))
    }
    return retTasks
  }

  activeChanged(activeIds: Task[]): void {
    this.$emit('activeChanged', activeIds[0])
  }
}
</script>
