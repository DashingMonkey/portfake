<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSettingsStore } from '../../stores/settings'
import { useServerStore } from '../../stores/server'
import { useTabsStore } from '../../stores/tabs'
import { useToast } from '../../composables/useToast'
import i18n from '../../i18n'

const emit = defineEmits<{
  openSettings: []
}>()

const settingsStore = useSettingsStore()
const serverStore = useServerStore()
const tabsStore = useTabsStore()
const { show: showToast } = useToast()

const appWindow = getCurrentWindow()

async function minimizeWindow() {
  await appWindow.minimize()
}

async function toggleMaximizeWindow() {
  await appWindow.toggleMaximize()
}

async function closeWindow() {
  await appWindow.close()
}

async function toggleServer() {
  if (serverStore.isRunning) {
    try {
      await serverStore.stopServer()
      showToast({ message: i18n.global.t('server.stopped'), type: 'info' })
    } catch (e) {
      showToast({ message: i18n.global.t('server.stopFailed', { error: String(e) }), type: 'error' })
    }
  } else {
    serverStore.port = settingsStore.serverPort
    const origins = settingsStore.parsedCorsOrigins
    try {
      await serverStore.startServer(origins.length > 0 ? origins : ['*'], tabsStore.drafts)
      showToast({ message: i18n.global.t('server.started', { port: serverStore.port }), type: 'success' })
    } catch (e) {
      showToast({ message: i18n.global.t('server.startFailed', { error: String(e) }), type: 'error' })
    }
  }
}
</script>

<template>
  <header
    class="h-10 flex items-center justify-between px-3 border-b border-border-default bg-surface-deep select-none [app-region:drag] [-webkit-app-region:drag]"
    data-tauri-drag-region
  >
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-2">
        <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="4" width="18" height="16" rx="3"/>
          <path d="M8 10h8"/>
          <circle cx="12" cy="15" r="2"/>
        </svg>
        <span class="text-sm font-semibold text-text-primary tracking-wider">{{ $t('app.title') }}</span>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <div class="flex items-center gap-1.5 group relative">
        <span
          class="w-1.5 h-1.5 rounded-full shrink-0"
          :class="serverStore.isRunning ? 'bg-success' : 'bg-error'"
        />
        <span class="text-xs" :class="serverStore.isRunning ? 'text-success' : 'text-error'">{{ $t('server.label') }}</span>
        <div
          v-if="serverStore.isRunning"
          class="absolute left-0 top-full mt-1 px-2 py-1 bg-surface-elevated border border-border-default rounded-sm text-xs text-text-primary whitespace-nowrap opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-150 z-50 pointer-events-none"
        >
          {{ $t('server.port', { port: serverStore.port }) }}
        </div>
      </div>

      <button
        @click="toggleServer"
        class="w-10 px-2 py-0.5 text-xs rounded-sm bg-surface-elevated text-text-secondary hover:bg-surface-base transition-colors [app-region:no-drag] [-webkit-app-region:no-drag]"
      >
        {{ serverStore.isRunning ? $t('server.stop') : $t('server.start') }}
      </button>
      <!-- Settings -->
      <button
        @click="emit('openSettings')"
        class="w-7 h-7 flex items-center justify-center rounded-sm hover:bg-surface-elevated text-text-muted hover:text-text-primary transition-colors [app-region:no-drag] [-webkit-app-region:no-drag]"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
          <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
        </svg>
      </button>

      <!-- Theme Toggle -->
      <button
        @click="settingsStore.toggleTheme"
        class="relative flex items-center justify-between h-6 w-12 px-0.5 rounded-full bg-surface-elevated border border-border-default transition-all duration-200 [app-region:no-drag] [-webkit-app-region:no-drag]"
      >
        <svg class="w-3.5 h-3.5 text-warning ml-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd"/>
        </svg>
        <svg class="w-3.5 h-3.5 text-text-muted mr-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"/>
        </svg>
        <span
          class="absolute top-1/2 -translate-y-1/2 w-5 h-5 rounded-full bg-surface-base border border-border-default shadow-xs transition-all duration-200"
          :class="settingsStore.theme === 'dark' ? 'left-[26px]' : 'left-px'"
          style="transition-property: left, transform; will-change: left"
        />
      </button>

      <!-- Window Controls -->
      <div class="flex items-center gap-1 ml-1">
        <button
          @click="minimizeWindow"
          class="w-6 h-6 flex items-center justify-center rounded-sm hover:bg-surface-elevated text-text-muted hover:text-text-primary transition-colors [app-region:no-drag] [-webkit-app-region:no-drag]"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path d="M20 12H4"/>
          </svg>
        </button>
        <button
          @click="toggleMaximizeWindow"
          class="w-6 h-6 flex items-center justify-center rounded-sm hover:bg-surface-elevated text-text-muted hover:text-text-primary transition-colors [app-region:no-drag] [-webkit-app-region:no-drag]"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path d="M8 3v3a2 2 0 01-2 2H3m18 0h-3a2 2 0 01-2-2V3m0 18v-3a2 2 0 012-2h3M3 16h3a2 2 0 012 2v3"/>
          </svg>
        </button>
        <button
          @click="closeWindow"
          class="w-6 h-6 flex items-center justify-center rounded-sm hover:bg-error/10 text-text-muted hover:text-error transition-colors [app-region:no-drag] [-webkit-app-region:no-drag]"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </div>
  </header>
</template>
