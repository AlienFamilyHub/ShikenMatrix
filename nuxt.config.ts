import { fileURLToPath } from "node:url";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
	compatibilityDate: "2025-07-15",
	devtools: { enabled: true },
	modules: ["@nuxt/icon", "@nuxt/fonts"],
	srcDir: "src/",
	alias: {
		"@": fileURLToPath(new URL("./src", import.meta.url)),
	},
	typescript: {
		strict: true,
	},
	routeRules: {
		"/**": { prerender: true },
	},
	ssr: false,
	devServer: {
		host: "0.0.0.0",
		port: 3002,
	},
	vite: {
		clearScreen: false,
		envPrefix: ["VITE_", "TAURI_"],
		server: {
			strictPort: true,
		},
	},
	ignore: ["**/src-tauri/**"],
});
