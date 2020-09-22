<script lang="ts">
import {LMap, LMarker, LTileLayer, LControlLayers} from "vue2-leaflet";
import {baseUrl} from "@/environment";
import "leaflet/dist/leaflet.css";
import L from 'leaflet';
import Vue from "vue";
import {FactorioBot} from "@/factorio-bot/bot";
import {Position} from "@/factorio-bot/types";

const xFactor = 3
const yFactor = -3

export default Vue.extend({
  name: "Map",
  components: {
    LMap,
    LTileLayer,
    LMarker,
    LControlLayers
  },
  methods: {
    mapReady() {
      const lmap = this.$refs.lmap as any
      this.$data.map = lmap.mapObject as L.Map
    },
    position2latlng(position: Position): L.LatLng {
      return L.latLng(position.y / yFactor, position.x / xFactor)
    },
    latlng2position(latlng: L.LatLng): Position {
      return {x: latlng.lng * xFactor, y: latlng.lat * yFactor}
    },
    async onClickMap(event: any) {
      const w = window as any
      const bot: FactorioBot = w.bots.bots[0]
      const position = this.latlng2position(event.latlng)
      console.log('move to', event.latlng, '/', position)
      await bot.move(position, 1)
    },
  },
  data(): { [name: string]: string } {
    const w = window as any
    return {
      baseUrl,
      bots: w.bots,
    };
  },
});
</script>

<template>
  <v-row>
    <v-btn v-on:click="bots.updatePlayers()" style="text-align: center">Update Bots</v-btn>
    <LMap ref="lmap" @ready="mapReady()"
          style="height: 80vh; width: 100vw;"
          @click="onClickMap"
          :minZoom="1"
          :maxZoom="8">
      <LControlLayers position="topright"  ></LControlLayers>
      <LTileLayer name="map" :url="`${baseUrl}/api/tiles/{z}/{x}/{y}/map_tile.png`" :noWrap="true" layer-type="base"></LTileLayer>
      <LTileLayer name="schematic" :url="`${baseUrl}/api/tiles/{z}/{x}/{y}/schematic_tile.png`" :noWrap="true" layer-type="base" :visible="false"></LTileLayer>
      <LTileLayer name="blocked" :url="`${baseUrl}/api/tiles/{z}/{x}/{y}/blocked_tile.png`" :noWrap="true" layer-type="overlay" :visible="false"></LTileLayer>
      <LTileLayer name="resources" :url="`${baseUrl}/api/tiles/{z}/{x}/{y}/resource_tile.png`" :noWrap="true" layer-type="overlay" :visible="false"></LTileLayer>
      <LTileLayer name="entities" :url="`${baseUrl}/api/tiles/{z}/{x}/{y}/entity_graph_tile.png`" :noWrap="true" layer-type="overlay" :visible="false"></LTileLayer>
      <LTileLayer name="flow" :url="`${baseUrl}/api/tiles/{z}/{x}/{y}/flow_graph_tile.png`" :noWrap="true" layer-type="overlay" :visible="false"></LTileLayer>

      <LMarker v-for="player in $store.state.players" v-bind:key="player.playerId" :lat-lng="position2latlng(player.position)"/>
    </LMap>
  </v-row>
</template>
