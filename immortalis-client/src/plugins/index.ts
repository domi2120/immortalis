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
import { createI18n } from 'vue-i18n';
import { messages } from '@/lang/en'; // Translations

// Types
import type { App } from 'vue'

const i18n = createI18n({
  locale: navigator.language.slice(0,2), // set locale
  legacy: false,
  fallbackLocale: 'en', // set fallback locale
  messages: {...messages, ...(await import(`@/lang/${navigator.language.slice(0, 2)}.ts`)).messages} // dynamically load the users language in addition to english which is the fallback language
})

export function registerPlugins (app: App) {
  loadFonts()
  app
    .use(vuetify)
    .use(router)
    .use(pinia)
    .use(i18n)
}
