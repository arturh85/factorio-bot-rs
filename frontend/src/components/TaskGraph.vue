<template>
    <v-card style="overflow-y: scroll; height: 85vh;" ref="card">
      <v-card-title>Tasks</v-card-title>
      <div id="tasksGraph2" style="height: 700px"></div>
    </v-card>
</template>

<script lang="ts">
import Vue from "vue";
import Component from 'vue-class-component'
import {Task, TaskStatus} from "@/factorio-bot/task";
import {formatDuration} from "@/factorio-bot/util";
import {State} from "@/store";


export default Vue.extend({
  name: "TaskGraph",
  mounted() {
    this.$store.subscribe((mutation, state: State) => {
      if (mutation.type == 'updateTaskGraphDot') {
        const plan = state.taskGraphDot;
        const d3 = (window as any).d3;
        (window as any).colors = d3.schemeCategory20;
        d3.select("#tasksGraph2").graphviz({fit: true, width: '100%', height: '100%'}).renderDot(plan)
            // .transition(function () {
            //   return d3.transition("main")
            //       .ease(d3.easeLinear)
            //       .delay(500)
            //       .duration(1500);
            // })
            .logEvents(true)
            .on("initEnd", () => {
              console.log("init end");

            });
      }
    })
  }
});

</script>
