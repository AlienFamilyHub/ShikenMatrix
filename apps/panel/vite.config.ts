import { fileURLToPath, URL } from "node:url";
import vue from "@vitejs/plugin-vue";
import Icons from "unplugin-icons/vite";
import { defineConfig } from "vite";

// The panel is served by the Rust server as embedded static files, so every
// API call is same-origin in production. In dev the Vite server proxies /api
// to a locally running server instead of relying on cross-origin CORS.
export default defineConfig({
  plugins: [
    vue({
      features: { vapor: true },
    }),
    Icons({
      scale: 1,
      // Icons compile to plain, static SVG templates so they stay Vapor
      // components. Width/height attributes are dropped (CSS sizes them) and
      // every icon is decorative: the adjacent label carries the meaning.
      compiler: {
        extension: "vue",
        compiler: (svg: string) =>
          `<template>\n${svg
            .replace("<svg", `<svg aria-hidden="true" focusable="false"`)
            .replace(/\s(?:width|height)="[^"]*"/g, "")}\n</template>\n`,
      },
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  server: {
    port: 4400,
    strictPort: true,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:4317",
        changeOrigin: false,
      },
    },
  },
  preview: {
    port: 4400,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:4317",
        changeOrigin: false,
      },
    },
  },
  build: {
    target: "es2022",
    assetsDir: "assets",
  },
});
