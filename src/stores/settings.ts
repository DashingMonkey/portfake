import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { Theme } from './types'
import i18n from '../i18n'
import type { Language } from '../i18n'

const SETTINGS_KEY = 'portfake-settings'

interface SavedSettings {
  theme: Theme
  serverPort: number
  corsOrigins: string
  language: Language
}

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<Theme>('light')
  const serverPort = ref(3210)
  const corsOrigins = ref('*')
  const language = ref<Language>(
    (i18n.global.locale as unknown as { value: string }).value.startsWith('zh') ? 'zh' : 'en'
  )

  function loadSettings() {
    const saved = localStorage.getItem(SETTINGS_KEY)
    if (saved) {
      try {
        const parsed: SavedSettings = JSON.parse(saved)
        if (parsed.theme === 'dark' || parsed.theme === 'light') {
          theme.value = parsed.theme
        }
        if (typeof parsed.serverPort === 'number' && parsed.serverPort > 0 && parsed.serverPort < 65536) {
          serverPort.value = parsed.serverPort
        }
        if (typeof parsed.corsOrigins === 'string') {
          corsOrigins.value = parsed.corsOrigins
        }
        if (parsed.language === 'en' || parsed.language === 'zh') {
          language.value = parsed.language
          ;(i18n.global.locale as unknown as { value: Language }).value = parsed.language
        }
      } catch {
        // ignore
      }
    } else {
      const legacyTheme = localStorage.getItem('portfake-theme')
      if (legacyTheme === 'dark' || legacyTheme === 'light') {
        theme.value = legacyTheme
      }
    }
    applyTheme()
  }

  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
    applyTheme()
  }

  function applyTheme() {
    document.documentElement.classList.toggle('dark', theme.value === 'dark')
  }

  function setLanguage(lang: Language) {
    language.value = lang
    ;(i18n.global.locale as unknown as { value: Language }).value = lang
  }

  function saveSettings() {
    const data: SavedSettings = {
      theme: theme.value,
      serverPort: serverPort.value,
      corsOrigins: corsOrigins.value,
      language: language.value,
    }
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(data))
  }

  watch(theme, saveSettings)
  watch(serverPort, saveSettings)
  watch(corsOrigins, saveSettings)
  watch(language, saveSettings)

  return {
    theme,
    serverPort,
    corsOrigins,
    language,
    loadSettings,
    toggleTheme,
    applyTheme,
    setLanguage,
    saveSettings,
  }
})
