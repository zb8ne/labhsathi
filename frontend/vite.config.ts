import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // Local dev only -- Compose/kind/Railway set VITE-independent runtime
      // config via env.js (see index.html + docker/entrypoint.sh) instead.
      "/api": "http://localhost:8080",
    },
  },
});
