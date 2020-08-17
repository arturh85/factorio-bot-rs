<template>
  <v-card>
    <v-card-title>Factorio Bot</v-card-title>
    <v-card-text>
      <!-- Page Content -->
      <v-btn v-on:click="bots.researchAutomation()">Research automation</v-btn>
<!--      <v-btn v-on:click="bots.researchRocketsilo()">Research rocket-silo</v-btn>-->
      <v-btn v-on:click="bots.updatePlayers()">Update Bots</v-btn>
      <v-btn v-on:click="bots.saveWorldAndServer()">Save World & Level</v-btn>
<!--      <br/>-->
<!--      <br/>-->
<!--      <v-btn v-on:click="bots.testCheatStuff()">testCheatStuff</v-btn>-->
<!--      <v-btn v-on:click="bots.testOffshorePump()">Test Offshore Pump</v-btn>-->
<!--      <v-btn color="primary" v-on:click="bots.buildStarterBase()">Build Starter Base</v-btn>-->
<!--      <v-btn v-on:click="bots.testBuildIronMinerSmelter(1)">Build IronMinerSmelter x1</v-btn>-->
<!--      <v-btn v-on:click="bots.testBuildIronMinerSmelter(5)">Build IronMinerSmelter x5</v-btn>-->
<!--      <v-btn v-on:click="bots.testBuildCopperMinerSmelter(1)">Build CopperMinerSmelter x1</v-btn>-->
<!--      <v-btn v-on:click="bots.testBuildStoneMinerChest(1)">Build StoneMinerChest x1</v-btn>-->
<!--      <v-btn v-on:click="bots.testBuildCoalLoop(2)">Build Coal Loop x2</v-btn>-->
<!--      <v-btn v-on:click="bots.testGatherTask('coal', 30)">Gather coal x30</v-btn>-->
<!--      <v-btn v-on:click="bots.testGatherTask('stone', 30)">Gather stone x30</v-btn>-->
<!--      <v-btn v-on:click="bots.testGatherTask('iron-plate', 20)">Gather iron-plate x20</v-btn>-->
<!--      <v-btn v-on:click="bots.testCraftTask('steam-engine', 2)">Craft steam-engine x2</v-btn>-->
<!--      <br/>-->
<!--      <br/>-->
<!--      <v-btn color="primary" v-on:click="bots.processTasks()">Process Tasks</v-btn>-->
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
// @ is an alias to /src
import Vue from "vue";
import {FactorioApi} from "@/factorio-bot/restApi";
import {FactorioBotManager} from "@/factorio-bot/bot-manager";

export default Vue.extend({
  name: "Control",
  components: {
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
