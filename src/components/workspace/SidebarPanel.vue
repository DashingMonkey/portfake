<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCollectionsStore } from '../../stores/collections'
import { useTabsStore } from '../../stores/tabs'
import { useServerStore } from '../../stores/server'
import { useToast } from '../../composables/useToast'
import type { Request } from '../../stores/types'
import ConfirmDialog from '../common/ConfirmDialog.vue'

const { t } = useI18n()

defineProps<{
  panel: 'collections' | 'logs'
}>()

const collectionsStore = useCollectionsStore()
const tabsStore = useTabsStore()
const serverStore = useServerStore()
const { show: showToast } = useToast()

// New Collection
const showNewCollection = ref(false)
const newCollectionName = ref('')

// Rename Collection
const showRenameCollection = ref(false)
const renameCollectionId = ref('')
const renameCollectionName = ref('')

// Collection context menu
const collectionContextMenu = ref<{ show: boolean; x: number; y: number; collectionId: string | null }>({
  show: false,
  x: 0,
  y: 0,
  collectionId: null,
})

// Request context menu
const requestContextMenu = ref<{ show: boolean; x: number; y: number; requestId: string | null; requestName: string }>({
  show: false,
  x: 0,
  y: 0,
  requestId: null,
  requestName: '',
})

// Delete confirm dialogs
const deleteCollectionTarget = ref<{ id: string; name: string } | null>(null)
const deleteRequestTarget = ref<{ id: string; name: string } | null>(null)

const requestsByCollection = computed(() => {
  const map: Record<string, Request[]> = {}
  for (const r of collectionsStore.requests) {
    if (!map[r.collection_id]) map[r.collection_id] = []
    map[r.collection_id].push(r)
  }
  return map
})

// Logs expand state
const expandedLogId = ref<string | null>(null)

function toggleLogExpand(id: string) {
  expandedLogId.value = expandedLogId.value === id ? null : id
}

function decodeQuery(query: string): string {
  try {
    return decodeURIComponent(query)
  } catch {
    return query
  }
}

// Sync sidebar with active tab
watch(() => tabsStore.activeTabId, () => {
  syncSidebarWithActiveTab()
})

/** Expand the collection containing the active request and scroll it into view */
async function syncSidebarWithActiveTab() {
  const activeTab = tabsStore.activeTab
  if (!activeTab?.requestId) {
    collectionsStore.selectedRequestId = null
    return
  }

  collectionsStore.selectedRequestId = activeTab.requestId

  // Check if request is already in loaded requests
  let request = collectionsStore.requests.find(r => r.id === activeTab.requestId)
  if (request) {
    if (!collectionsStore.expandedCollectionIds.has(request.collection_id)) {
      await collectionsStore.toggleExpand(request.collection_id)
    }
    return
  }

  // Use draft state to find collection_id without expanding all collections
  const draft = tabsStore.drafts.find(d => d.tabId === activeTab.id)
  const collectionId = draft?.state.collectionId
  if (collectionId && !collectionsStore.expandedCollectionIds.has(collectionId)) {
    await collectionsStore.toggleExpand(collectionId)
  }
}

function openSavedRequestTab(request: Request) {
  const tab = tabsStore.createTab(request.id, request.method, request.name)
  tabsStore.createDraft(tab.id, {
    collectionId: request.collection_id,
    name: request.name,
    method: request.method,
    path: request.path,
    statusCode: 200,
    headers: request.headers,
    body: '',
    bodyType: 'json',
    delayMs: '',
  })
  collectionsStore.selectedRequestId = request.id

  const collectionId = request.collection_id
  if (!collectionsStore.expandedCollectionIds.has(collectionId)) {
    collectionsStore.toggleExpand(collectionId)
  } else {
    collectionsStore.loadRequests(collectionId)
  }
}

function getMethodColor(method: string): string {
  switch (method.toUpperCase()) {
    case 'GET': return 'bg-success/15 text-success'
    case 'POST': return 'bg-info/15 text-info'
    case 'PUT': return 'bg-warning/15 text-warning'
    case 'PATCH': return 'bg-purple-400/15 text-purple-400'
    case 'DELETE': return 'bg-error/15 text-error'
    default: return 'bg-text-muted/15 text-text-muted'
  }
}

function getStatusColor(status: number): string {
  if (status < 300) return 'text-success'
  if (status < 400) return 'text-warning'
  return 'text-error'
}

async function confirmDeleteCollection() {
  const target = deleteCollectionTarget.value
  if (!target) return
  await collectionsStore.deleteCollection(target.id)
  showToast({ message: t('collection.deleted', { name: target.name }), type: 'info' })
  deleteCollectionTarget.value = null
}

async function handleCreateCollection() {
  const name = newCollectionName.value.trim()
  if (!name) return
  try {
    await collectionsStore.createCollection(name)
    showToast({ message: t('collection.created', { name }), type: 'success' })
    newCollectionName.value = ''
    showNewCollection.value = false
  } catch (e) {
    showToast({ message: t('collection.createFailed', { error: String(e) }), type: 'error' })
  }
}

function handleContextRename() {
  const collectionId = collectionContextMenu.value.collectionId
  if (collectionId) {
    const coll = collectionsStore.collections.find(c => c.id === collectionId)
    if (coll) {
      renameCollectionId.value = collectionId
      renameCollectionName.value = coll.name
      showRenameCollection.value = true
    }
  }
  closeCollectionContextMenu()
}

function handleContextDeleteCollection() {
  const collectionId = collectionContextMenu.value.collectionId
  if (collectionId) {
    const coll = collectionsStore.collections.find(c => c.id === collectionId)
    if (coll) {
      closeCollectionContextMenu()
      deleteCollectionTarget.value = { id: collectionId, name: coll.name }
    }
  }
}

async function handleRenameCollection() {
  const name = renameCollectionName.value.trim()
  if (!name || !renameCollectionId.value) return
  try {
    await collectionsStore.renameCollection(renameCollectionId.value, name)
    showToast({ message: t('collection.renamed', { name }), type: 'success' })
    renameCollectionName.value = ''
    renameCollectionId.value = ''
    showRenameCollection.value = false
  } catch (e) {
    showToast({ message: t('collection.renameFailed', { error: String(e) }), type: 'error' })
  }
}

function handleCollectionContextMenu(event: MouseEvent, collectionId: string) {
  event.preventDefault()
  event.stopPropagation()
  collectionContextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    collectionId,
  }
}

function closeCollectionContextMenu() {
  collectionContextMenu.value.show = false
}

function handleContextNewRequest() {
  const collectionId = collectionContextMenu.value.collectionId
  if (collectionId) {
    if (!collectionsStore.expandedCollectionIds.has(collectionId)) {
      collectionsStore.toggleExpand(collectionId)
    }
    const tab = tabsStore.createTab('', 'GET', t('request.newRequest'))
    tabsStore.createDraft(tab.id, { collectionId, name: '', method: 'GET', path: '', statusCode: 200, headers: '[\n  {\n    "key": "Content-Type",\n    "value": "application/json",\n    "enabled": true\n  }\n]', body: '{\n  "success": true\n}', bodyType: 'json', delayMs: '' })
  }
  closeCollectionContextMenu()
}

function handleRequestContextMenu(event: MouseEvent, req: Request) {
  event.preventDefault()
  event.stopPropagation()
  requestContextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    requestId: req.id,
    requestName: req.name,
  }
}

function closeRequestContextMenu() {
  requestContextMenu.value.show = false
}

function handleDeleteRequest() {
  const { requestId, requestName } = requestContextMenu.value
  if (!requestId) return
  closeRequestContextMenu()
  deleteRequestTarget.value = { id: requestId, name: requestName }
}

async function confirmDeleteRequest() {
  const target = deleteRequestTarget.value
  if (!target) return
  try {
    await collectionsStore.deleteRequest(target.id)
    const tab = tabsStore.tabs.find(t => t.requestId === target.id)
    if (tab) {
      tabsStore.closeTab(tab.id)
    }
    showToast({ message: t('collection.requestDeleted', { name: target.name }), type: 'info' })
  } catch (e) {
    showToast({ message: t('collection.requestDeleteFailed', { error: String(e) }), type: 'error' })
  }
  deleteRequestTarget.value = null
}

onMounted(async () => {
  await collectionsStore.loadCollections()
})
</script>

<template>
  <div class="h-full flex flex-col">
    <template v-if="panel === 'collections'">
      <div class="px-3 py-2 border-b border-border-default flex items-center justify-between">
        <span class="text-[11px] font-semibold tracking-[0.08em] text-text-secondary uppercase">{{ t('collection.collections') }}</span>
        <button
          @click="showNewCollection = true"
          class="p-1 text-text-muted hover:text-accent rounded-md hover:bg-surface-elevated transition-colors duration-150"
          :title="t('collection.newCollection')"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>
      <div class="flex-1 overflow-auto p-2 space-y-1">
        <div v-if="collectionsStore.loadError" class="p-3 text-center text-error text-xs">
          {{ t('collection.loadFailed', { error: collectionsStore.loadError }) }}
        </div>
        <div
          v-for="collection in collectionsStore.collections"
          :key="collection.id"
          class="rounded-sm"
        >
          <div
            class="flex items-center gap-1.5 px-2 py-1.5 text-xs font-medium text-text-primary hover:bg-surface-elevated rounded-sm cursor-pointer group"
            :class="{ 'opacity-40': !collection.enabled }"
            @click="collectionsStore.toggleExpand(collection.id)"
            @contextmenu="handleCollectionContextMenu($event, collection.id)"
          >
            <svg
              class="w-3 h-3 text-text-muted transition-transform shrink-0"
              :class="collectionsStore.isExpanded(collection.id) ? 'rotate-90' : ''"
              fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5"
            >
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7"/>
            </svg>
            <span class="flex-1 truncate">{{ collection.name }}</span>
            <div
              class="relative w-6 h-3.5 rounded-full transition-colors duration-150 shrink-0 cursor-pointer"
              :class="collection.enabled ? 'bg-accent' : 'bg-border-default'"
              @click.stop="collectionsStore.setCollectionEnabled(collection.id, !collection.enabled)"
              :title="collection.enabled ? t('collection.disable') : t('collection.enable')"
            >
              <span
                class="absolute top-0.5 left-0.5 w-2.5 h-2.5 bg-white rounded-full transition-transform duration-150"
                :class="collection.enabled ? 'translate-x-2.5' : 'translate-x-0'"
              />
            </div>
          </div>

          <div v-if="collectionsStore.isExpanded(collection.id)" class="ml-5 space-y-0.5">
            <div
              v-for="req in requestsByCollection[collection.id] || []"
              :key="req.id"
              class="flex items-center gap-1.5 px-2 py-1 text-xs rounded-sm cursor-pointer hover:bg-surface-elevated group"
              :class="[
                collectionsStore.selectedRequestId === req.id && 'bg-surface-elevated',
                !req.enabled && 'opacity-40'
              ]"
              @click="openSavedRequestTab(req)"
              @contextmenu="handleRequestContextMenu($event, req)"
            >
              <span class="text-[10px] font-semibold px-1 py-0.5 rounded-sm shrink-0" :class="getMethodColor(req.method)">
                {{ req.method }}
              </span>
              <span class="flex-1 truncate">{{ req.name }}</span>
              <div
                class="relative w-6 h-3.5 rounded-full transition-colors duration-150 shrink-0 cursor-pointer"
                :class="req.enabled ? 'bg-accent' : 'bg-border-default'"
                @click.stop="collectionsStore.setRequestEnabled(req.id, !req.enabled)"
                :title="req.enabled ? t('collection.disableReq') : t('collection.enableReq')"
              >
                <span
                  class="absolute top-0.5 left-0.5 w-2.5 h-2.5 bg-white rounded-full transition-transform duration-150"
                  :class="req.enabled ? 'translate-x-2.5' : 'translate-x-0'"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template v-if="panel === 'logs'">
      <div class="px-3 py-2 border-b border-border-default flex items-center justify-between">
        <span class="text-[11px] font-semibold tracking-[0.08em] text-text-secondary uppercase">{{ t('logs.requestLogs') }}</span>
        <button
          @click="serverStore.clearLogs"
          class="text-[10px] text-text-muted hover:text-text-primary transition-colors"
        >
          {{ t('logs.clear') }}
        </button>
      </div>
      <div class="flex-1 overflow-auto">
        <div v-if="serverStore.logs.length === 0" class="p-4 text-center text-text-muted text-xs">
          {{ t('collection.noRequests') }}
        </div>
        <div v-else class="divide-y divide-border-default/50">
          <div
            v-for="log in serverStore.logs"
            :key="log.id"
            class="text-xs"
          >
            <!-- Summary row -->
            <div class="px-3 py-2 hover:bg-surface-elevated/50 cursor-pointer" @click="toggleLogExpand(log.id)">
              <div class="flex items-center gap-2">
                <span class="text-text-muted font-mono">{{ log.time }}</span>
                <span class="font-semibold px-1 py-0.5 rounded-sm text-xs" :class="getMethodColor(log.method)">
                  {{ log.method }}
                </span>
                <span class="font-mono font-semibold text-xs" :class="getStatusColor(log.status)">{{ log.status }}</span>
                <svg
                  class="w-3 h-3 text-text-muted transition-transform ml-auto"
                  :class="expandedLogId === log.id ? 'rotate-90' : ''"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7"/>
                </svg>
              </div>
              <div class="text-text-secondary truncate mt-0.5">{{ log.path }}</div>
            </div>

            <div
              v-if="expandedLogId === log.id"
              class="px-3 pb-3 space-y-2 border-t border-border-default/50 bg-surface-base select-text"
            >
              <div v-if="log.query" class="pt-2">
                <div class="text-[10px] text-text-muted uppercase tracking-wide mb-0.5">{{ t('logs.query') }}</div>
                <div class="text-text-secondary font-mono text-[10px] break-all">{{ decodeQuery(log.query) }}</div>
              </div>

              <div v-if="log.headers && log.headers.length" class="pt-2">
                <div class="text-[10px] text-text-muted uppercase tracking-wide mb-0.5">{{ t('logs.headers') }}</div>
                <div class="space-y-0.5">
                  <div
                    v-for="([k, v], i) in log.headers"
                    :key="i"
                    class="flex gap-1.5 text-[10px]"
                  >
                    <span class="text-text-secondary font-mono shrink-0">{{ k }}:</span>
                    <span class="text-text-primary font-mono break-all">{{ v }}</span>
                  </div>
                </div>
              </div>

              <div v-if="log.body" class="pt-2">
                <div class="text-[10px] text-text-muted uppercase tracking-wide mb-0.5">{{ t('logs.body') }}</div>
                <pre class="text-text-secondary font-mono text-[10px] bg-surface-deep p-2 rounded-sm overflow-auto max-h-40">{{ log.body }}</pre>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <div
      v-if="showNewCollection"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      @click.self="showNewCollection = false"
    >
      <div class="bg-surface-deep border border-border-default rounded-lg shadow-xl w-80">
        <div class="px-4 py-3 border-b border-border-default">
          <span class="text-sm font-semibold text-text-primary">{{ t('collection.newCollection') }}</span>
        </div>
        <div class="p-4 space-y-3">
          <input
            v-model="newCollectionName"
            :placeholder="t('collection.collectionName')"
            class="w-full px-3 py-2 text-sm bg-surface-base border border-border-default rounded-sm text-text-primary placeholder:text-text-muted focus:outline-hidden focus:border-accent"
            @keyup.enter="handleCreateCollection"
          />
          <div class="flex justify-end gap-2">
            <button
              @click="showNewCollection = false"
              class="px-3 py-1.5 text-xs text-text-secondary hover:text-text-primary hover:bg-surface-elevated rounded-sm transition-colors"
            >
              {{ t('collection.cancel') }}
            </button>
            <button
              @click="handleCreateCollection"
              class="px-3 py-1.5 text-xs bg-accent text-white rounded-sm hover:bg-accent/90 transition-colors"
            >
              {{ t('collection.create') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="showRenameCollection"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      @click.self="showRenameCollection = false"
    >
      <div class="bg-surface-deep border border-border-default rounded-lg shadow-xl w-80">
        <div class="px-4 py-3 border-b border-border-default">
          <span class="text-sm font-semibold text-text-primary">{{ t('collection.renameCollection') }}</span>
        </div>
        <div class="p-4 space-y-3">
          <input
            v-model="renameCollectionName"
            :placeholder="t('collection.collectionName')"
            class="w-full px-3 py-2 text-sm bg-surface-base border border-border-default rounded-sm text-text-primary placeholder:text-text-muted focus:outline-hidden focus:border-accent"
            @keyup.enter="handleRenameCollection"
          />
          <div class="flex justify-end gap-2">
            <button
              @click="showRenameCollection = false"
              class="px-3 py-1.5 text-xs text-text-secondary hover:text-text-primary hover:bg-surface-elevated rounded-sm transition-colors"
            >
              {{ t('collection.cancel') }}
            </button>
            <button
              @click="handleRenameCollection"
              class="px-3 py-1.5 text-xs bg-accent text-white rounded-sm hover:bg-accent/90 transition-colors"
            >
              {{ t('collection.rename') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="collectionContextMenu.show"
        class="fixed z-50 bg-surface-base border border-border-default rounded-lg shadow-lg py-1 min-w-[160px]"
        :style="{ left: `${collectionContextMenu.x}px`, top: `${collectionContextMenu.y}px` }"
        @click.stop
      >
        <button
          class="w-full px-4 py-2 text-left text-xs text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="handleContextNewRequest"
        >
          {{ t('collection.newRequest') }}
        </button>
        <button
          class="w-full px-4 py-2 text-left text-xs text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors duration-150"
          @click="handleContextRename"
        >
          {{ t('collection.rename') }}
        </button>
        <button
          class="w-full px-4 py-2 text-left text-xs text-error hover:bg-surface-elevated hover:text-error transition-colors duration-150"
          @click="handleContextDeleteCollection"
        >
          {{ t('collection.delete') }}
        </button>
      </div>
      <div
        v-if="collectionContextMenu.show"
        class="fixed inset-0 z-40"
        @click="closeCollectionContextMenu"
        @contextmenu.prevent="closeCollectionContextMenu"
      ></div>
    </Teleport>

    <Teleport to="body">
      <div
        v-if="requestContextMenu.show"
        class="fixed z-50 bg-surface-base border border-border-default rounded-lg shadow-lg py-1 min-w-[160px]"
        :style="{ left: `${requestContextMenu.x}px`, top: `${requestContextMenu.y}px` }"
        @click.stop
      >
        <button
          class="w-full px-4 py-2 text-left text-xs text-error hover:bg-surface-elevated hover:text-error transition-colors duration-150"
          @click="handleDeleteRequest"
        >
          {{ t('collection.delete') }}
        </button>
      </div>
      <div
        v-if="requestContextMenu.show"
        class="fixed inset-0 z-40"
        @click="closeRequestContextMenu"
        @contextmenu.prevent="closeRequestContextMenu"
      ></div>
    </Teleport>

    <ConfirmDialog
      :visible="!!deleteCollectionTarget"
      :title="t('collection.deleteCollectionTitle')"
      :message="deleteCollectionTarget ? t('collection.deleteCollectionMessage', { name: deleteCollectionTarget.name }) : ''"
      :confirm-text="t('common.delete')"
      danger
      @confirm="confirmDeleteCollection"
      @cancel="deleteCollectionTarget = null"
    />

    <ConfirmDialog
      :visible="!!deleteRequestTarget"
      :title="t('collection.deleteRequestTitle')"
      :message="deleteRequestTarget ? t('collection.deleteRequestMessage', { name: deleteRequestTarget.name }) : ''"
      :confirm-text="t('common.delete')"
      danger
      @confirm="confirmDeleteRequest"
      @cancel="deleteRequestTarget = null"
    />
  </div>
</template>
