/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import '@mdi/font/css/materialdesignicons.css'
import 'vuetify/styles'
import { VDataTable } from "vuetify/labs/VDataTable";

// Composables
import { createVuetify } from 'vuetify'


// Translations provided by Vuetify
import { en, de } from 'vuetify/locale'

// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
export default createVuetify({
  locale: {
    locale: navigator.language.slice(0,2),
    fallback: "en",
    messages: { en, de }
  },
  components: {
    VDataTable
  },
  theme: {
    defaultTheme: "dark"
  },
})
