<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCollectionsStore } from '../../stores/collections'
import { useTabsStore } from '../../stores/tabs'
import { useServerStore } from '../../stores/server'
import SidebarPanel from './SidebarPanel.vue'
import RequestEditor from './RequestEditor.vue'

const { t } = useI18n()

const collectionsStore = useCollectionsStore()
const tabsStore = useTabsStore()
const serverStore = useServerStore()

const activePanel = ref<'collections' | 'logs' | null>('collections')

function togglePanel(panel: 'collections' | 'logs') {
  activePanel.value = activePanel.value === panel ? null : panel
}

// Sidebar resize
const SIDEBAR_WIDTH_KEY = 'portfake-sidebar-width'
const sidebarWidth = ref(256)
const isResizing = ref(false)

function startResize(e: MouseEvent) {
  e.preventDefault()
  isResizing.value = true
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
}

function onResize(e: MouseEvent) {
  if (!isResizing.value) return
  const newWidth = Math.min(window.innerWidth * 0.7, Math.max(160, e.clientX - 48))
  sidebarWidth.value = newWidth
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
  localStorage.setItem(SIDEBAR_WIDTH_KEY, String(sidebarWidth.value))
}

onMounted(async () => {
  await collectionsStore.loadCollections()
  serverStore.loadStatus()
  tabsStore.loadTabs()
  const savedWidth = parseInt(localStorage.getItem(SIDEBAR_WIDTH_KEY) || '', 10)
  if (!isNaN(savedWidth) && savedWidth >= 160) sidebarWidth.value = savedWidth
})

onUnmounted(() => {
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
})
</script>

<template>
  <div class="h-full flex">
    <!-- Activity Bar -->
    <div class="w-12 h-full flex flex-col items-center py-2 gap-1 bg-surface-deep border-r border-border-default shrink-0">
      <button
        @click="togglePanel('collections')"
        class="w-10 h-10 flex items-center justify-center rounded-md transition-colors duration-150 group relative"
        :class="activePanel === 'collections'
          ? 'bg-surface-active text-white'
          : 'text-text-secondary hover:bg-surface-elevated hover:text-text-primary'"
        title="Collections"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/>
        </svg>
        <div class="absolute left-full ml-2 px-2 py-1 bg-surface-elevated border border-border-default rounded-sm text-xs text-text-primary whitespace-nowrap opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-150 z-50 pointer-events-none">
          {{ t('workspace.collections') }}
        </div>
      </button>

      <button
        @click="togglePanel('logs')"
        class="w-10 h-10 flex items-center justify-center rounded-md transition-colors duration-150 group relative"
        :class="activePanel === 'logs'
          ? 'bg-surface-active text-white'
          : 'text-text-secondary hover:bg-surface-elevated hover:text-text-primary'"
        title="Request Logs"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
        </svg>
        <div class="absolute left-full ml-2 px-2 py-1 bg-surface-elevated border border-border-default rounded-sm text-xs text-text-primary whitespace-nowrap opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-150 z-50 pointer-events-none">
          {{ t('workspace.requestLogs') }}
        </div>
      </button>

      <div class="flex-1" />
    </div>

    <!-- Sidebar Panel -->
    <div
      v-if="activePanel"
      class="bg-surface-deep flex flex-col shrink-0"
      :style="{ width: sidebarWidth + 'px' }"
    >
      <SidebarPanel :panel="activePanel" />
    </div>

    <!-- Resize Handle -->
    <div
      v-if="activePanel"
      class="relative z-10 select-none cursor-col-resize shrink-0"
      style="width: 1px; background: transparent"
      @mousedown="startResize"
    >
      <div
        class="absolute left-0 top-0 bottom-0 w-px transition-all duration-150"
        :style="isResizing ? 'background-color: rgba(0, 122, 204, 0.5); width: 2px' : 'background-color: var(--border-default)'"
      />
      <div class="absolute -left-2 -right-2 top-0 bottom-0" :class="{ 'cursor-col-resize': isResizing }" />
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex flex-col bg-surface-base min-w-0 min-h-0">
      <RequestEditor />
    </div>
  </div>
</template>
