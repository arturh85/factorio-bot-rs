<template>
  <v-card>
    <v-card-title>Factorio Bot</v-card-title>
    <v-card-text>
      <!-- Page Content -->
      <v-menu offset-y>
        <template v-slot:activator="{ on, attrs }">
          <v-btn
              color="primary"
              dark
              v-bind="attrs"
              v-on="on"
          >
            Start
          </v-btn>
        </template>
        <v-list>
          <v-list-item v-on:click="bots.startRocket()">
            <v-list-item-title>Start Rocket</v-list-item-title>
          </v-list-item>
<!--          <v-list-item v-on:click="bots.researchAutomation()">-->
<!--            <v-list-item-title>Research automation</v-list-item-title>-->
<!--          </v-list-item>-->
<!--          <v-list-item v-on:click="bots.researchLogistics()">-->
<!--            <v-list-item-title>Research logistics</v-list-item-title>-->
<!--          </v-list-item>-->
          <v-list-item v-on:click="bots.researchSteelProcessing()">
            <v-list-item-title>Research steel-processing</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.researchLogisticSciencePack()">
            <v-list-item-title>Research logistic-science-pack</v-list-item-title>
          </v-list-item>
<!--          <v-list-item v-on:click="bots.researchRocketsilo()">-->
<!--            <v-list-item-title>Research rocket-silo</v-list-item-title>-->
<!--          </v-list-item>-->
          <v-list-item v-on:click="bots.testCheatStuff()">
            <v-list-item-title>testCheatStuff</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.buildStarterBase()">
            <v-list-item-title>Build Starter Base</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildIronMinerSmelter(1)">
            <v-list-item-title>Build IronMinerSmelter x1</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildIronMinerSmelter(5)">
            <v-list-item-title>Build IronMinerSmelter x5</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildCopperMinerSmelter(1)">
            <v-list-item-title>Build CopperMinerSmelter x1</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildStoneMinerChest(1)">
            <v-list-item-title>Build StoneMinerChest x1</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildCoalLoop(2)">
            <v-list-item-title>Build Coal Loop x2</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildFurnaceLine(Entities.ironOre, Entities.ironPlate)">
            <v-list-item-title>Build FurnaceLine iron-plate</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildFurnaceLine(Entities.copperOre, Entities.copperPlate)">
            <v-list-item-title>Build FurnaceLine copper-plate</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildFurnaceLine(Entities.stone, Entities.stoneBrick)">
            <v-list-item-title>Build FurnaceLine stone-brick</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildMinerLine(Entities.ironOre)">
            <v-list-item-title>Build MinerLine iron-ore</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildMinerLine(Entities.copperOre)">
            <v-list-item-title>Build MinerLine copper-ore</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildMinerLine(Entities.coal)">
            <v-list-item-title>Build MinerLine coal</v-list-item-title>
          </v-list-item>
          <v-list-item v-on:click="bots.testBuildMinerLine(Entities.stone)">
            <v-list-item-title>Build MinerLine stone</v-list-item-title>
          </v-list-item>

          <v-list-item v-bind:key="name" v-for="(count, name) in {
        [Entities.coal]: 30,
        [Entities.stone]: 30,
        [Entities.ironPlate]: 30,
      }" v-on:click="bots.testGatherTask(name, count)">
            <v-list-item-title>Gather {{ name }} x{{ count }}</v-list-item-title>
          </v-list-item>

          <v-list-item v-bind:key="name" v-for="(count, name) in {
        [Entities.smallElectricPole]: 2,
        [Entities.electricMiningDrill]: 2,
      }" v-on:click="bots.testCraftTask(name, count)">
            <v-list-item-title>Craft {{ name }} x{{ count }}</v-list-item-title>
          </v-list-item>


        </v-list>
      </v-menu>
      &nbsp;
      <v-btn v-on:click="bots.saveWorldAndServer()">Save</v-btn>

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
  components: {},
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
