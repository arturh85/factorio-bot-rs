import Vue from "vue";
import VueRouter, {RouteConfig} from "vue-router";
import Home from "../views/Home.vue";
import EntityGraph from "@/views/EntityGraph.vue";
import TasksGraph from "@/views/TasksGraph.vue";
import FlowGraph from "@/views/FlowGraph.vue";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
    {
        path: "/",
        name: "Control",
        component: Home,
    },
    {
        path: "/tasks",
        name: "TasksGraph",
        component: TasksGraph,
    },
    {
        path: "/entities",
        name: "EntityGraph",
        component: EntityGraph,
    },
    {
        path: "/flow",
        name: "FlowGraph",
        component: FlowGraph,
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
