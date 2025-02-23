import Aura from '@primevue/themes/aura';

export default defineNuxtConfig({
  app: {
    head: {
      title: "Mockomatic",
    },
    link: [{ rel: "icon", type: "image/x-icon", href: "/favicon.ico" }],
  },
  compatibilityDate: '2024-11-01',
  devtools: { enabled: true },
  css: [
    "@/assets/css/main.css",
    "primeicons/primeicons.css"
  ],
  modules: [
    '@primevue/nuxt-module',
    '@pinia/nuxt',
  ],
  primevue: {
    importTheme: { from: '@/themes/custom_theme.js' },
    options: {
      ripple: false,
      inputVariant: 'filled',
      autoImport: true,
      theme: {
        preset: Aura,
        options: {
          prefix: 'p',
          darkModeSelector: 'system',
          cssLayer: false
        }
      }
    }
  },
  runtimeConfig: {
    public: {
      apiBase: 'http://localhost:8080/api/v1',
    },
  },
})
