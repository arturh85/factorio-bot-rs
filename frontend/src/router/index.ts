import Vue from "vue";
import VueRouter, {RouteConfig} from "vue-router";
import Home from "../views/Home.vue";
import Graph from "@/views/Graph.vue";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
    {
        path: "/",
        name: "Control",
        component: Home,
    },
    {
        path: "/graph",
        name: "Graph",
        component: Graph,
    },
    {
        path: "/map",
        name: "Map",
        // route level code-splitting
        // this generates a separate chunk (about.[hash].js) for this route
        // which is lazy-loaded when the route is visited.
        component: () =>
            import(/* webpackChunkName: "about" */ "../views/Map.vue"),
    },
];

const router = new VueRouter({
    routes,
});

export default router;
