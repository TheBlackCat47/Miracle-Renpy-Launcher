import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  // ID_client is a public OAuth client identifier. Do not expose other .env keys.
  envPrefix: ['VITE_', 'ID_'],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
});
