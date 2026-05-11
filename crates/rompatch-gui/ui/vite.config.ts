import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri dev sets TAURI_DEV_HOST when running on a physical device. For
// our macOS-only desktop case it's always unset, so the host is local.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  // Tauri expects a fixed port; fail loudly if it's already in use.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: 'ws', host, port: 1421 }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**', '**/target/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
});
