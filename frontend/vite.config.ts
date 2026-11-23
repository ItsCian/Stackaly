import path from 'node:path';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import tailwindcss from '@tailwindcss/vite';
import { tanstackRouter } from '@tanstack/router-plugin/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    // Compiles messages/*.json into typed functions in src/paraglide
    paraglideVitePlugin({
      project: './project.inlang',
      outdir: './src/paraglide',
      emitTsDeclarations: true,
      // Locale: the cookie first (set from the user's profile at login),
      // otherwise French. Never from the URL (STA-46)
      strategy: ['cookie', 'baseLocale'],
    }),
    // Generates routeTree.gen.ts: must run before react()
    tanstackRouter({ target: 'react', autoCodeSplitting: true }),
    react(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(import.meta.dirname, './src'),
    },
  },
  server: {
    proxy: {
      '/api': { target: 'http://127.0.0.1:3000' },
    },
  },
});
