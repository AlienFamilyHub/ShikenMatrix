import process from "node:process";
import Icons from "unplugin-icons/vite";
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

const serverTarget = process.env.SHIKENMATRIX_SERVER_ADDR ?? "127.0.0.1:4317";

// https://vite.dev/config/
export default defineConfig({
  plugins: [solid(), tailwindcss(), Icons({ compiler: "solid" })],

  clearScreen: false,

  server: {
    port: 1430,
    strictPort: true,
    proxy: {
      "/api": { target: `http://${serverTarget}`, changeOrigin: true },
      "/health": { target: `http://${serverTarget}`, changeOrigin: true },
    },
  },
});
