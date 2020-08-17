<template>
  <v-container>
    <v-row>
      <v-col>
        <Control />
      </v-col>
    </v-row>
    <v-row>
      <v-col>
        <Tasks :tasks="$store.state.tasks" @activeChanged="activeChanged" />
      </v-col>
<!--      <v-col>-->
<!--        <SelectedTask :task="$store.state.selectedTask" />-->
<!--      </v-col>-->
      <v-col>
        <v-card>
          <v-card-title>Bots: </v-card-title>
          <Player v-for="playerId in Object.keys(players)" v-bind:key="playerId"
                  :player="$store.getters.getPlayer(playerId)"/>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
// @ is an alias to /src
import Vue from "vue";
import Player from "@/components/Player.vue";
import Tasks from "@/components/Tasks.vue";
import {FactorioApi} from "@/factorio-bot/restApi";
import {FactorioBotManager} from "@/factorio-bot/bot-manager";
import {Task} from "@/factorio-bot/task";
// import SelectedTask from "@/components/SelectedTask.vue";
import Control from "@/components/Control.vue";

export default Vue.extend({
  name: "Home",
  components: {
    // SelectedTask,
    Player,
    Control,
    Tasks
  },
  methods: {
    activeChanged(task: Task) {
      this.$store.commit('changeSelectedTask', task ? task : null)
    }
  },
  computed: {
    players() {
      return this.$store.state.players
    }
  },
  data(): { [name: string]: any } {
    const w = window as any
    return {
      FactorioApi,
      bots: w.bots as FactorioBotManager,
    }
  },
});
</script>
