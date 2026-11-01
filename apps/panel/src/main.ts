import { createVaporApp } from "vue";
import App from "./app.vue";
import { startRouter } from "./lib/router";

startRouter();
createVaporApp(App).mount("#app");
