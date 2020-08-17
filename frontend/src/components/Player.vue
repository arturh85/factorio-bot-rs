<template>
  <v-card>
    <v-card-title>Bot #{{ player.playerId }} @ {{ positionLabel(player.position) }}</v-card-title>
    <v-list-item v-for="itemName in Object.keys(player.mainInventory)" v-bind:key="itemName">
      <v-list-item-content>
        <v-list-item-title>
          {{ itemName }} x {{ player.mainInventory[itemName] }}
        </v-list-item-title>
      </v-list-item-content>
    </v-list-item>
  </v-card>
</template>

<script lang="ts">
import Vue from "vue";
import Component from 'vue-class-component'
import {positionLabel} from "@/factorio-bot/util";
import {Position} from "@/factorio-bot/types";

// Define the props by using Vue's canonical way.
const PlayerProps = Vue.extend({
  props: {
    player: Object
  }
})

@Component
export default class Player extends PlayerProps {
  // created() {
  //   console.log('PLAYER', this.player, Object.keys(this.player.mainInventory))
  // }
  positionLabel(position: Position): string {
    return positionLabel(position)
  }
}
</script>
