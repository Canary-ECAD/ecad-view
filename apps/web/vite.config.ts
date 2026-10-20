import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";
import { rmSync } from "node:fs";
import { resolve } from "node:path";

// VITE_NO_SAMPLES=1: build sin ficheros de ejemplo (p. ej. view.covaga.dev).
const noSamples = process.env.VITE_NO_SAMPLES === "1";

export default defineConfig({
  // Versión de la app (package.json), mostrada en Ajustes.
  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version ?? "dev"),
  },
  plugins: [
    react(),
    // PWA: precachea todo el bundle (incluidos los .wasm de 7z y sql.js y los
    // ficheros de demo) para que el visor funcione completamente sin conexión.
    // El manifest vive en public/manifest.webmanifest.
    VitePWA({
      registerType: "autoUpdate",
      injectRegister: "auto",
      manifest: false,
      workbox: {
        globPatterns: [
          "**/*.{js,css,html,svg,png,wasm,woff2,webmanifest}",
          ...(noSamples ? [] : ["demo/*"]),
        ],
        maximumFileSizeToCacheInBytes: 8 * 1024 * 1024,
      },
    }),
    noSamples && {
      name: "strip-samples",
      apply: "build",
      writeBundle(options) {
        rmSync(resolve(options.dir ?? "dist", "demo"), { recursive: true, force: true });
      },
    },
  ],
  optimizeDeps: {
    // 7z-wasm es un módulo emscripten; el pre-bundling de esbuild lo rompe.
    exclude: ["7z-wasm"],
  },
  build: {
    target: "es2022",
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules/three")) return "three";
        },
      },
    },
  },
});
