<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import AppHeader from './components/layout/AppHeader.vue'
import MainWorkspace from './components/workspace/MainWorkspace.vue'
import SettingsDialog from './components/common/SettingsDialog.vue'
import { useSettingsStore } from './stores/settings'
import { useServerStore } from './stores/server'

const settingsStore = useSettingsStore()
const serverStore = useServerStore()
const isInitializing = ref(true)
const showSettings = ref(false)

/** Initialize app state: load settings, configure server port, and set up event listeners */
async function initializeApp() {
  settingsStore.loadSettings()
  serverStore.port = settingsStore.serverPort
  await serverStore.setupListener()
  isInitializing.value = false
}

onMounted(initializeApp)
onUnmounted(() => {
  serverStore.cleanupListener()
})
</script>

<template>
  <div class="h-screen flex flex-col bg-surface-deep text-text-primary overflow-hidden">
    <AppHeader
      @open-settings="showSettings = true"
    />
    <main class="flex-1 overflow-auto">
      <div v-if="isInitializing" class="h-full flex items-center justify-center">
        <div class="text-text-muted text-sm">{{ $t('app.loading') }}</div>
      </div>
      <MainWorkspace v-else />
    </main>
    <SettingsDialog v-model:visible="showSettings" />
  </div>
</template>
