import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import i18n from './i18n'
import './style.css'

/** Prevent default context menu except on elements marked with data-contextmenu */
function onContextMenu(e: Event) {
  const target = e.target as HTMLElement
  if (!target.closest('[data-contextmenu]')) {
    e.preventDefault()
  }
}

const app = createApp(App)
app.use(i18n)
app.use(createPinia())

document.addEventListener('contextmenu', onContextMenu)
// Clean up the global listener when the app unmounts
app.mount('#app')
window.addEventListener('beforeunload', () => {
  document.removeEventListener('contextmenu', onContextMenu)
})
