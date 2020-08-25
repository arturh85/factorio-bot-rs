<template>
  <v-app>
    <v-app-bar app color="primary" dark>
      <div class="d-flex align-center">
        <v-img
            alt="Vuetify Logo"
            class="shrink mr-2"
            contain
            src="/logo.png"
            transition="scale-transition"
            width="40"
        />

        <v-toolbar-title>Factorio Bot</v-toolbar-title>
      </div>

      <v-spacer></v-spacer>
      <router-link to="/">
        <v-btn text>
          <span class="mr-2">Control</span>
        </v-btn>
      </router-link>
      <router-link to="/map">
        <v-btn text>
          <span class="mr-2">Map</span>
        </v-btn>
      </router-link>
    </v-app-bar>

    <v-main>
      <router-view></router-view>
    </v-main>
  </v-app>
</template>

<script lang="ts">
import Vue from "vue";
import {FactorioBotManager} from "@/factorio-bot/bot-manager";
import {FactorioApi} from "@/factorio-bot/restApi";

export default Vue.extend({
  name: "App",

  components: {
  },

  data: () => ({
    //
  }),


  created() {
    const manager: FactorioBotManager = new FactorioBotManager(this.$store)
    const w = window as any
    manager.init().then(bots => {
      for (let i = 0; i < bots.length; i++) {
        const name = 'bot' + (i > 0 ? i + 1 : '')
        w[name] = bots[i]
      }
    })
    w.bots = manager
    w.api = FactorioApi

    const ws = new WebSocket('ws://localhost:7123/ws/');
    ws.onmessage = (evt: MessageEvent) => {
      console.log('WS MESSAGE', evt);
    };
  },
  beforeDestroy() {

  }


});
</script>
