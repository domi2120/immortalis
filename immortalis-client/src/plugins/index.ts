/**
 * plugins/index.ts
 *
 * Automatically included in `./src/main.ts`
 */

// Plugins
import { loadFonts } from './webfontloader'
import vuetify from './vuetify'
import pinia from '../store'
import router from '../router'

// Types
import type { App } from 'vue'

import { createI18n } from 'vue-i18n';

const i18n = createI18n({
  locale: navigator.language, // set locale
  fallbackLocale: 'en', // set fallback locale
  messages: {
    en: {
      "search": "Search"
    },
    ja: {
      "search": "探す"
    },
    de: {
      "search": "Suchen"
    }
  }
})

export function registerPlugins (app: App) {
  loadFonts()
  app
    .use(vuetify)
    .use(router)
    .use(pinia)
    .use(i18n)
}
