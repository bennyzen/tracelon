// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-05-15',
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  ssr: false,
  vite: {
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_'],
    server: { strictPort: true },
    optimizeDeps: {
      include: [
        '@tauri-apps/plugin-dialog',
        '@tauri-apps/api/window',
        '@tauri-apps/api/core',
        'svgo',
      ],
    },
  },
  ignore: ['**/src-tauri/**'],
})
