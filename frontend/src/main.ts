import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import store from "./store";
import vuetify from "./plugins/vuetify";

// Vue.config.productionTip = false;
import { Icon } from 'leaflet';

type D = Icon.Default & {
    _getIconUrl: string;
};
Icon.Default.mergeOptions({
    iconRetinaUrl: require('leaflet/dist/images/marker-icon-2x.png'),
    iconUrl: require('leaflet/dist/images/marker-icon.png'),
    shadowUrl: require('leaflet/dist/images/marker-shadow.png'),
});


delete (Icon.Default.prototype as D)._getIconUrl;



new Vue({
    router,
    store,
    vuetify,
    render: (h) => h(App),
}).$mount("#app");
