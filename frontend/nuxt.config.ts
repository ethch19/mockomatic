import tailwindcss from '@tailwindcss/vite'

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
    "~/assets/css/main.css",
    "~/assets/css/tailwind.css",
  ],
  typescript: {
    typeCheck: true,
    tsConfig: {
        compilerOptions: {
            strict: true
        }
    },
    nodeTsConfig: {
      compilerOptions: {
        strict: true
      }
    }
  },
  modules: [
    '@pinia/nuxt',
    'shadcn-nuxt',
    '@nuxtjs/color-mode',
    '@vee-validate/nuxt',
    'vue-sonner/nuxt',
  ],
  veeValidate: {
    autoImports: true,
    componentNames: {
        Field: 'FormField',
    },
  },
  vueSonner: {
    css: true, 
  },
  vite: {
    plugins: [
        tailwindcss(),
    ],
  },
  pinia: {
    storesDirs: ['./stores/**'],
  },
  shadcn: {
    prefix: '',
    componentDir: './components/ui',
  },
  colorMode: {
    classSuffix: '',
  },
  vue: {
    compilerOptions: {
        isCustomElement: (tag) => tag === "iconify-icon",
    },
  },
  runtimeConfig: {
    public: {
      apiBase: 'http://localhost:8080/api/v1', // change in deployment
    },
  },
})
