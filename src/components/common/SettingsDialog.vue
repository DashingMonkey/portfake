<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'
import { useServerStore } from '../../stores/server'
import { useTabsStore } from '../../stores/tabs'
import { useToast } from '../../composables/useToast'
import type { Language } from '../../i18n'
import pkg from '../../../package.json'

const appVersion = pkg.version
const { t } = useI18n()
const visible = defineModel<boolean>('visible')
const settingsStore = useSettingsStore()
const serverStore = useServerStore()
const tabsStore = useTabsStore()
const { show } = useToast()

const languages: { value: Language; label: string }[] = [
  { value: 'en', label: 'English' },
  { value: 'zh', label: '中文' },
]

async function handleSave() {
  if (settingsStore.serverPort < 1 || settingsStore.serverPort > 65535) {
    show({ message: t('settings.portRangeError'), type: 'error' })
    return
  }

  settingsStore.saveSettings()
  visible.value = false

  if (serverStore.isRunning) {
    const origins = settingsStore.parsedCorsOrigins
    try {
      await serverStore.stopServer()
      serverStore.port = settingsStore.serverPort
      await serverStore.startServer(origins.length > 0 ? origins : ['*'], tabsStore.drafts)
      show({ message: t('server.started', { port: serverStore.port }), type: 'success' })
    } catch (e) {
      show({ message: t('server.startFailed', { error: String(e) }), type: 'error' })
    }
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-150"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-90 flex items-center justify-center bg-black/40 backdrop-blur-xs"
        @click.self="visible = false"
        @keydown.esc="visible = false"
      >
        <div
          role="dialog"
          aria-modal="true"
          class="w-[400px] bg-surface-base border border-border-default rounded-xl shadow-2xl overflow-hidden"
        >
          <!-- Header -->
          <div class="px-5 py-4 border-b border-border-default flex items-center justify-between">
            <h2 class="text-sm font-semibold text-text-primary">{{ t('settings.title') }}</h2>
            <button
              @click="visible = false"
              class="w-6 h-6 flex items-center justify-center rounded-sm hover:bg-surface-elevated text-text-muted hover:text-text-primary transition-colors"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>

          <!-- Body -->
          <div class="px-5 py-4 space-y-4">
            <!-- Language -->
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm font-medium text-text-primary">{{ t('settings.language') }}</div>
              </div>
              <div class="flex gap-1">
                <button
                  v-for="lang in languages"
                  :key="lang.value"
                  @click="settingsStore.setLanguage(lang.value)"
                  class="px-2.5 py-1 text-xs rounded-sm transition-colors"
                  :class="settingsStore.language === lang.value
                    ? 'bg-accent/10 text-accent font-medium'
                    : 'text-text-secondary hover:bg-surface-elevated'"
                >
                  {{ lang.label }}
                </button>
              </div>
            </div>

            <!-- Port -->
            <div>
              <div class="text-sm font-medium text-text-primary">{{ t('settings.serverPort') }}</div>
              <div class="text-xs text-text-muted mt-0.5">{{ t('settings.serverPortDescription') }}</div>
              <input
                v-model.number="settingsStore.serverPort"
                type="number"
                min="1"
                max="65535"
                class="no-spin mt-2 w-full px-3 py-1.5 text-sm bg-surface-deep border border-border-default rounded-sm focus:outline-hidden focus:border-accent text-text-primary"
              />
            </div>

            <!-- CORS Origins -->
            <div>
              <div class="text-sm font-medium text-text-primary">{{ t('settings.corsOrigins') }}</div>
              <div class="text-xs text-text-muted mt-0.5">{{ t('settings.corsOriginsDescription') }}</div>
              <input
                v-model="settingsStore.corsOrigins"
                type="text"
                placeholder="*"
                class="mt-2 w-full px-3 py-1.5 text-sm bg-surface-deep border border-border-default rounded-sm focus:outline-hidden focus:border-accent text-text-primary"
              />
            </div>
          </div>

          <!-- Footer -->
          <div class="px-5 py-3 border-t border-border-default flex items-center justify-between">
            <span class="text-xs text-text-muted">v{{ appVersion }}</span>
            <button
              @click="handleSave"
              class="px-4 py-1.5 text-xs font-semibold bg-accent hover:bg-cyan-400 text-white rounded-sm transition-colors"
            >
              {{ t('settings.save') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.no-spin::-webkit-outer-spin-button,
.no-spin::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
.no-spin {
  -moz-appearance: textfield;
}
</style>
