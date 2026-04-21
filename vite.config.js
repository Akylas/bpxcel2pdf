import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { readdirSync } from 'fs';
import { join } from 'path';
import config from './package.json' with { type: 'json' };

const ignoreWarnings = new Set([
  'a11y-no-onchange',
  'a11y-label-has-associated-control',
  'a11y-mouse-events-have-key-events',
]);

export default defineConfig(({ mode }) => {
  const locales = readdirSync(join('src', 'i18n'))
    .filter((s) => s.endsWith('.json'))
    .map((s) => s.replace('.json', ''));
  const production = mode === 'production';
  return {
    root: './src',
    base: './', // use relative paths
    publicDir: '../public',
    clearScreen: false,
    server: {
      port: 3001,
      strictPort: true,
    },
    // to make use of `TAURI_ENV_*` env variables
    envPrefix: ['VITE_', 'TAURI_ENV_'],
    build: {
      outDir: '../build',
      emptyOutDir: true,
      // tauri supports es2021
      target: ['es2021', 'chrome100', 'safari15'],
      // don't minify for debug builds
      minify: !process.env.TAURI_ENV_DEBUG && 'esbuild',
      // produce sourcemaps for debug builds
      sourcemap: !!process.env.TAURI_ENV_DEBUG,
    },
    plugins: [
      svelte({
        onwarn(warning, defaultHandler) {
          if (ignoreWarnings.has(warning.code)) return;
          defaultHandler(warning);
        },
      }),
    ],
    define: {
      SUPPORTED_LOCALES: JSON.stringify(locales),
      REPO_URL: JSON.stringify(config.homepage),
      PRODUCTION: production,
    },
  };
});
