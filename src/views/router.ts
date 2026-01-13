import { createRouter, createWebHistory } from "vue-router";

const routes = [
	{ path: "/", component: () => import("./index.vue") }, // 首页
];

const router = createRouter({
	history: createWebHistory(),
	routes,
});

export default router;
