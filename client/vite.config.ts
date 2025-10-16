import { defineConfig } from "vite";

// https://vite.dev/config/
export default defineConfig({
  server: {
    proxy: { "/eval": "http://127.0.0.1:8080" },
  },
});
