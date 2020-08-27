<template>
  <v-card>
    <v-card-title>Factorio Bot</v-card-title>
    <v-card-text>
      <!-- Page Content -->
      <v-btn color="primary" v-on:click="bots.saveWorldAndServer()">Save</v-btn>
      <v-btn v-on:click="bots.researchAutomation()">Research automation</v-btn>
      <v-btn v-on:click="bots.researchLogistics()">Research logistics</v-btn>
      <v-btn v-on:click="bots.researchSteelProcessing()">Research steel-processing</v-btn>
      <v-btn v-on:click="bots.researchLogisticSciencePack()">Research logistic-science-pack</v-btn>
      <v-btn v-on:click="bots.researchRocketsilo()">Research rocket-silo</v-btn>
      <v-btn v-on:click="bots.testCheatStuff()">testCheatStuff</v-btn>
      <v-btn v-on:click="bots.buildStarterBase()">Build Starter Base</v-btn>
      <v-btn v-on:click="bots.testBuildIronMinerSmelter(1)">Build IronMinerSmelter x1</v-btn>
      <v-btn v-on:click="bots.testBuildIronMinerSmelter(5)">Build IronMinerSmelter x5</v-btn>
      <v-btn v-on:click="bots.testBuildCopperMinerSmelter(1)">Build CopperMinerSmelter x1</v-btn>
      <v-btn v-on:click="bots.testBuildStoneMinerChest(1)">Build StoneMinerChest x1</v-btn>
      <v-btn v-on:click="bots.testBuildCoalLoop(2)">Build Coal Loop x2</v-btn>

      <v-btn v-bind:key="name" v-for="(count, name) in {
        [Entities.coal]: 30,
        [Entities.stone]: 30,
        [Entities.ironPlate]: 30,
      }" v-on:click="bots.testGatherTask(name, count)">Gather  {{name}} x{{count}}</v-btn>

      <v-btn v-bind:key="name" v-for="(count, name) in {
        [Entities.smallElectricPole]: 2,
        [Entities.electricMiningDrill]: 2,
      }" v-on:click="bots.testCraftTask(name, count)">Craft  {{name}} x{{count}}</v-btn>

    </v-card-text>
  </v-card>
</template>

<script lang="ts">
// @ is an alias to /src
import Vue from "vue";
import {FactorioApi} from "@/factorio-bot/restApi";
import {FactorioBotManager} from "@/factorio-bot/bot-manager";
import {Entities} from "@/factorio-bot/types";

export default Vue.extend({
  name: "Control",
  components: {
  },
  data(): { [name: string]: any } {
    const w = window as any
    return {
      FactorioApi,
      bots: w.bots as FactorioBotManager,
      Entities: Entities,
    }
  },
});
</script>
