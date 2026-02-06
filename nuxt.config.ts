export default defineNuxtConfig({
  ssr: false,
  compatibilityDate: '2026-02-04',
  devtools: { enabled: true },
  experimental: {
    appManifest: true
  },
  app: {
    baseURL: './',
    head: {
      title: 'Email Client'
    }
  },
  css: ['~/assets/tailwind.css', '~/assets/main.css'],
  modules: ['@pinia/nuxt', '@nuxt/ui'],
  tailwindcss: {
    cssPath: '~/assets/tailwind.css'
  }
})
