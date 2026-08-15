<script setup lang="ts">
import { ref, reactive, watch, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from '@headlessui/vue'
import { useCollectionsStore } from '../../stores/collections'
import { useTabsStore } from '../../stores/tabs'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../../composables/useToast'
import TabBar from '../tabs/TabBar.vue'
import MethodSelect from '../common/MethodSelect.vue'
import { DEFAULT_MOCK_DRAFT_STATE, type Request } from '../../stores/types'

const { t } = useI18n()

/** State for textarea resize drag interaction */
interface ResizeState {
  isResizing: boolean
  startY: number
  startHeight: number
  textarea: HTMLTextAreaElement | null
}

const resizeState = reactive<ResizeState>({
  isResizing: false,
  startY: 0,
  startHeight: 0,
  textarea: null,
})

function startResize(event: MouseEvent, textarea: HTMLTextAreaElement) {
  resizeState.isResizing = true
  resizeState.startY = event.clientY
  resizeState.startHeight = textarea.clientHeight
  resizeState.textarea = textarea
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
}

function onResize(event: MouseEvent) {
  if (!resizeState.isResizing || !resizeState.textarea) return
  const delta = event.clientY - resizeState.startY
  const newHeight = Math.max(80, resizeState.startHeight + delta)
  resizeState.textarea.style.height = `${newHeight}px`
}

function stopResize() {
  resizeState.isResizing = false
  resizeState.textarea = null
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
}

const collectionsStore = useCollectionsStore()
const tabsStore = useTabsStore()
const { show: showToast } = useToast()

/** Currently active saved request ID, null if new unsaved tab */
const activeRequestId = computed(() => tabsStore.activeTab?.requestId || null)
/** Whether the current tab is an unsaved/new request */
const isUnsavedTab = computed(() => !!tabsStore.activeTab && !tabsStore.activeTab.requestId)

const activeRequestName = computed(() => {
  if (!tabsStore.activeTab) return ''
  const tab = tabsStore.activeTab
  if (!tab.requestId) {
    const draft = tabsStore.drafts.find(d => d.tabId === tab.id)
    return draft?.state.name || tab.title || t('request.newRequest')
  }
  const req = collectionsStore.requests.find(r => r.id === tab.requestId)
  return req?.name || tab.title || ''
})

const unsavedForm = reactive({ ...DEFAULT_MOCK_DRAFT_STATE })
const isLoadingDraft = ref(false)
const isLoadingEditForm = ref(false)
const duplicateError = ref<string | null>(null)

const isEditingTitle = ref(false)
const editedTitle = ref('')

const editForm = reactive({
  collectionId: '',
  exampleId: '',
  method: 'GET',
  path: '',
  statusCode: 200,
  headers: '[]',
  body: '',
  bodyType: 'json',
  delayMs: '',
})

const isEditingRequestName = ref(false)
const editedRequestName = ref('')

const unsavedCollectionName = computed(() => {
  if (!unsavedForm.collectionId) return ''
  const found = collectionsStore.collections.find(c => c.id === unsavedForm.collectionId)
  return found ? found.name : ''
})

const editCollectionName = computed(() => {
  if (!editForm.collectionId) return ''
  const found = collectionsStore.collections.find(c => c.id === editForm.collectionId)
  return found ? found.name : ''
})

/** Sync form state when switching between tabs to preserve unsaved changes */
watch(() => tabsStore.activeTabId, async (newTabId, oldTabId) => {
  duplicateError.value = null

  // Save current form state to draft before switching
  if (oldTabId) {
    const oldTab = tabsStore.tabs.find(t => t.id === oldTabId)
    const oldDraft = tabsStore.drafts.find(d => d.tabId === oldTabId)
    if (oldDraft) {
      if (oldTab?.requestId) {
        tabsStore.updateDraftState(oldDraft.id, {
          collectionId: editForm.collectionId,
          name: oldDraft.state.name,
          method: editForm.method,
          path: editForm.path,
          statusCode: editForm.statusCode,
          headers: editForm.headers,
          body: editForm.body,
          bodyType: editForm.bodyType,
          delayMs: editForm.delayMs,
        })
      } else {
        tabsStore.updateDraftState(oldDraft.id, { ...unsavedForm })
      }
    }
  }

  if (!newTabId) {
    collectionsStore.examples = []
    isEditingTitle.value = false
    isEditingRequestName.value = false
    return
  }

  const tab = tabsStore.activeTab
  if (!tab) {
    isEditingTitle.value = false
    isEditingRequestName.value = false
    return
  }

  // Load appropriate form state based on tab type (new vs saved request)
  if (!tab.requestId) {
    isLoadingDraft.value = true
    const draft = tabsStore.drafts.find(d => d.tabId === tab.id)
    if (draft) {
      Object.assign(unsavedForm, draft.state)
    } else {
      Object.assign(unsavedForm, DEFAULT_MOCK_DRAFT_STATE)
    }
    isEditingTitle.value = false
    isEditingRequestName.value = false
    nextTick(() => { isLoadingDraft.value = false })
  } else {
    const draft = tabsStore.drafts.find(d => d.tabId === tab.id)
    if (draft && draft.changesCount > 0) {
      // Restore unsaved changes from draft
      isLoadingEditForm.value = true
      editForm.collectionId = draft.state.collectionId || ''
      editForm.exampleId = ''
      editForm.method = draft.state.method
      editForm.path = draft.state.path
      editForm.statusCode = draft.state.statusCode
      editForm.headers = draft.state.headers
      editForm.body = draft.state.body
      editForm.bodyType = draft.state.bodyType
      editForm.delayMs = draft.state.delayMs
      nextTick(() => { isLoadingEditForm.value = false })
      collectionsStore.loadExamples(tab.requestId)
    } else {
      // Load fresh data from backend
      isLoadingEditForm.value = true
      const req = collectionsStore.requests.find(r => r.id === tab.requestId)
      editForm.collectionId = req?.collection_id || ''
      editForm.method = tab.method
      editForm.path = draft?.state.path || ''
      nextTick(() => { isLoadingEditForm.value = false })
      collectionsStore.loadExamples(tab.requestId)
    }
    isEditingTitle.value = false
    isEditingRequestName.value = false
  }
}, { immediate: true })

/** Auto-save unsaved form changes to draft state */
watch(unsavedForm, (newState) => {
  if (isLoadingDraft.value) return
  if (tabsStore.activeTab && !tabsStore.activeTab.requestId) {
    const draft = tabsStore.drafts.find(d => d.tabId === tabsStore.activeTab!.id)
    if (draft) {
      tabsStore.updateDraftState(draft.id, { ...newState })
      tabsStore.markDirty(draft.id)
    }
  }
}, { deep: true })

watch(editForm, () => {
  if (isLoadingEditForm.value) return
  if (tabsStore.activeTab?.requestId) {
    const draft = tabsStore.drafts.find(d => d.tabId === tabsStore.activeTab!.id)
    if (draft) {
      tabsStore.markDirty(draft.id)
    }
  }
}, { deep: true })

watch(() => collectionsStore.examples, (examples) => {
  if (!tabsStore.activeTab || !tabsStore.activeTab.requestId) return
  const defaultEx = examples.find(e => e.is_default) || examples[0]
  if (defaultEx) {
    isLoadingEditForm.value = true
    editForm.exampleId = defaultEx.id
    editForm.statusCode = defaultEx.status_code
    editForm.headers = defaultEx.headers
    editForm.body = defaultEx.body
    editForm.bodyType = defaultEx.body_type
    editForm.delayMs = defaultEx.delay_ms || ''
    nextTick(() => { isLoadingEditForm.value = false })
  }
}, { deep: true })

watch(() => collectionsStore.requests, () => {
  if (!activeRequestId.value || editForm.collectionId) return
  const req = collectionsStore.requests.find(r => r.id === activeRequestId.value)
  if (req?.collection_id) {
    isLoadingEditForm.value = true
    editForm.collectionId = req.collection_id
    nextTick(() => { isLoadingEditForm.value = false })
  }
})

function startTitleEdit() {
  isEditingTitle.value = true
  editedTitle.value = unsavedForm.name
}

function finishTitleEdit() {
  const trimmed = editedTitle.value.trim()
  if (trimmed) {
    unsavedForm.name = trimmed
    const tab = tabsStore.activeTab
    if (tab) {
      tabsStore.updateTabTitle(tab.id, trimmed)
    }
  }
  isEditingTitle.value = false
}

function startRequestNameEdit() {
  isEditingRequestName.value = true
  editedRequestName.value = activeRequestName.value
}

async function finishRequestNameEdit() {
  const trimmed = editedRequestName.value.trim()
  if (trimmed && activeRequestId.value) {
    try {
      const req = collectionsStore.requests.find(r => r.id === activeRequestId.value)
      if (req) {
        await invoke('update_request', {
          requestId: req.id,
          collectionId: req.collection_id,
          name: trimmed,
          method: req.method,
          path: req.path,
          headers: req.headers,
        })
        req.name = trimmed
        const tab = tabsStore.activeTab
        if (tab) {
          tabsStore.updateTabTitle(tab.id, trimmed)
        }
        showToast({ message: t('request.nameUpdated'), type: 'success' })
      }
    } catch (e) {
      showToast({ message: t('request.nameUpdateFailed', { error: String(e) }), type: 'error' })
    }
  }
  isEditingRequestName.value = false
}

/** Save a new request to the selected collection */
async function handleCreateRequest() {
  const { collectionId, name, method, path, statusCode, headers, body, bodyType, delayMs } = unsavedForm

  if (!collectionId) {
    showToast({ message: t('request.selectCollectionError'), type: 'error' })
    return
  }
  if (!name.trim() || !path.trim()) {
    showToast({ message: t('request.namePathRequired'), type: 'error' })
    return
  }

  let exampleBody = body
  if (bodyType === 'json' && body.trim()) {
    try {
      exampleBody = JSON.stringify(JSON.parse(body), null, 2)
    } catch {
      // keep original if invalid JSON
    }
  }

  try {
    const request = await invoke<Request>('create_request', {
      collectionId,
      name: name.trim(),
      method,
      path: path.trim(),
      exampleStatusCode: statusCode,
      exampleHeaders: headers,
      exampleBody,
      exampleBodyType: bodyType,
      exampleDelayMs: delayMs || null,
    })

    const tab = tabsStore.activeTab
    if (tab) {
      tabsStore.bindTabToRequest(tab.id, request.id, method, name.trim())
      collectionsStore.selectedRequestId = request.id
      const draft = tabsStore.drafts.find(d => d.tabId === tab.id)
      if (draft) {
        tabsStore.markClean(draft.id)
      }
    }

    // Reset editForm for the newly created request to avoid stale data from previous tabs
    isLoadingEditForm.value = true
    editForm.collectionId = request.collection_id
    editForm.method = request.method
    editForm.path = request.path
    editForm.exampleId = ''
    editForm.statusCode = 200
    editForm.headers = '[]'
    editForm.body = ''
    editForm.bodyType = 'json'
    editForm.delayMs = ''
    nextTick(() => { isLoadingEditForm.value = false })

    await collectionsStore.loadCollections()
    await collectionsStore.loadRequests(collectionId)
    if (!collectionsStore.expandedCollectionIds.has(collectionId)) {
      collectionsStore.toggleExpand(collectionId)
    }
    await collectionsStore.loadExamples(request.id)

    showToast({ message: t('request.saved', { name: name.trim() }), type: 'success' })
  } catch (e) {
    showToast({ message: t('request.saveFailed', { error: String(e) }), type: 'error' })
  }
}

/** Check if a method+path combination already exists in DB or other tabs */
async function validateDuplicate(method: string, path: string, excludeRequestId?: string): Promise<boolean> {
  const trimmedPath = path.trim()
  if (!trimmedPath) {
    duplicateError.value = null
    return false
  }

  // Check against database
  const dbDuplicate = await invoke<boolean>('check_duplicate_request', {
    method: method.toUpperCase(),
    path: trimmedPath,
    excludeId: excludeRequestId || null,
  })
  if (dbDuplicate) {
    duplicateError.value = t('request.duplicate', { method: method.toUpperCase(), path: trimmedPath })
    return true
  }

  // Check against other unsaved tabs
  const currentTabId = tabsStore.activeTab?.id
  const draftDuplicate = tabsStore.drafts.find(d => {
    if (d.state.method.toUpperCase() !== method.toUpperCase()) return false
    if (d.state.path.trim() !== trimmedPath) return false
    const tab = tabsStore.tabs.find(t => t.id === d.tabId)
    if (!tab || tab.requestId === excludeRequestId) return false
    if (d.tabId === currentTabId) return false
    return true
  })
  if (draftDuplicate) {
    duplicateError.value = t('request.duplicateTab', { method: method.toUpperCase(), path: trimmedPath })
    return true
  }

  duplicateError.value = null
  return false
}

async function handleSavedPathCheck() {
  const path = editForm.path.trim()
  if (!path) return
  const req = collectionsStore.requests.find(r => r.id === activeRequestId.value)
  await validateDuplicate(editForm.method, path, req?.id)
}

function handleNewTab() {
  const tab = tabsStore.createTab('', 'GET', t('request.newRequest'))
  tabsStore.createDraft(tab.id, { ...DEFAULT_MOCK_DRAFT_STATE })
  isLoadingDraft.value = true
  Object.assign(unsavedForm, DEFAULT_MOCK_DRAFT_STATE)
  nextTick(() => { isLoadingDraft.value = false })
}

/** Update existing request metadata and its default example */
async function handleUpdateRequest() {
  const req = collectionsStore.requests.find(r => r.id === activeRequestId.value)
  if (!req) {
    showToast({ message: t('request.notFound'), type: 'error' })
    return
  }
  if (!editForm.collectionId) {
    showToast({ message: t('request.selectCollectionError'), type: 'error' })
    return
  }
  try {
    const methodChanged = req.method !== editForm.method
    const pathChanged = req.path !== editForm.path
    const collectionChanged = req.collection_id !== editForm.collectionId
    if (methodChanged || pathChanged || collectionChanged) {
      if (methodChanged || pathChanged) {
        if (await validateDuplicate(editForm.method, editForm.path, req.id)) return
      }
      const oldCollectionId = req.collection_id
      await invoke('update_request', {
        requestId: req.id,
        collectionId: editForm.collectionId,
        name: req.name,
        method: editForm.method,
        path: editForm.path,
        headers: req.headers,
      })
      req.method = editForm.method
      req.path = editForm.path
      req.collection_id = editForm.collectionId
      const tab = tabsStore.activeTab
      if (tab) {
        tab.method = editForm.method
        tabsStore.updateTabTitle(tab.id, req.name)
      }
      if (collectionChanged) {
        if (collectionsStore.expandedCollectionIds.has(oldCollectionId)) {
          await collectionsStore.loadRequests(oldCollectionId)
        }
        if (!collectionsStore.expandedCollectionIds.has(editForm.collectionId)) {
          await collectionsStore.toggleExpand(editForm.collectionId)
        }
        await collectionsStore.loadRequests(editForm.collectionId)
      }
    }

    await invoke('update_example', {
      exampleId: editForm.exampleId,
      statusCode: editForm.statusCode,
      headers: editForm.headers,
      body: editForm.body,
      bodyType: editForm.bodyType,
      delayMs: editForm.delayMs || null,
    })
    if (activeRequestId.value) {
      await collectionsStore.loadExamples(activeRequestId.value)
      const draft = tabsStore.drafts.find(d => d.tabId === tabsStore.activeTab?.id)
      if (draft) tabsStore.markClean(draft.id)
    }
    showToast({ message: t('request.updated'), type: 'success' })
  } catch (e) {
    showToast({ message: t('request.updateFailed', { error: String(e) }), type: 'error' })
  }
}

async function handleUnsavedPathCheck() {
  const path = unsavedForm.path.trim()
  if (!path) return
  await validateDuplicate(unsavedForm.method, path, undefined)
}

async function handleUnsavedMethodChange(method: string) {
  unsavedForm.method = method
  const path = unsavedForm.path.trim()
  if (path) {
    await validateDuplicate(method, path, undefined)
  }
}

function prettifyJson(jsonString: string): string {
  try {
    const parsed = JSON.parse(jsonString)
    return JSON.stringify(parsed, null, 2)
  } catch {
    showToast({ message: t('request.invalidJson'), type: 'error' })
    return jsonString
  }
}

function prettifyBody(form: { body: string; bodyType: string }) {
  if (form.bodyType !== 'json' || !form.body.trim()) return
  form.body = prettifyJson(form.body)
}

function prettifyHeaders(headersString: string): string {
  if (!headersString.trim()) return headersString
  return prettifyJson(headersString)
}

/** Handle Ctrl/Cmd+S keyboard shortcut to save current request */
function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault()
    if (isUnsavedTab.value) {
      if (!unsavedForm.collectionId || !unsavedForm.name.trim()) return
      handleCreateRequest()
    } else {
      handleUpdateRequest()
    }
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="flex-1 flex flex-col bg-surface-base min-w-0 min-h-0">
    <TabBar @new-tab="handleNewTab" />
    <div class="flex-1 overflow-auto p-4 min-h-0">
      <div v-if="!tabsStore.activeTab" class="flex items-center justify-center h-full">
        <div class="text-text-muted text-sm">{{ t('request.noTabSelected') }}</div>
      </div>

      <div v-else-if="isUnsavedTab" class="space-y-3">
        <div class="flex items-center gap-1">
          <div class="flex items-center gap-1 flex-1 min-w-0">
            <Listbox
              :model-value="unsavedForm.collectionId"
              @update:model-value="unsavedForm.collectionId = $event"
            >
              <div class="relative">
                <ListboxButton
                  class="text-xs bg-transparent border-none text-text-muted focus:outline-hidden cursor-pointer py-0.5 pr-1 flex items-center gap-0.5"
                >
                  <span class="truncate">{{ unsavedCollectionName || t('request.selectCollection') }}</span>
                  <svg class="w-3 h-3 opacity-60 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                  </svg>
                </ListboxButton>

                <transition
                  leave-active-class="transition duration-100 ease-in"
                  leave-from-class="opacity-100"
                  leave-to-class="opacity-0"
                >
                  <ListboxOptions
                    class="absolute z-10 mt-1 bg-surface-base shadow-xl max-h-60 rounded-md border border-border-default focus:outline-hidden overflow-hidden w-max"
                  >
                    <ListboxOption
                      v-for="c in collectionsStore.collections"
                      :key="c.id"
                      :value="c.id"
                      v-slot="{ active, selected }"
                    >
                      <li
                        class="cursor-pointer select-none relative py-1.5 pl-3 pr-9 transition-colors duration-100 text-xs whitespace-nowrap"
                        :class="[
                          active ? 'bg-accent/10 text-text-primary' : 'text-text-secondary',
                          selected ? 'font-semibold' : 'font-normal'
                        ]"
                      >
                        {{ c.name }}
                        <span v-if="selected" class="absolute inset-y-0 right-0 flex items-center pr-3 text-accent">
                          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                          </svg>
                        </span>
                      </li>
                    </ListboxOption>
                  </ListboxOptions>
                </transition>
              </div>
            </Listbox>
            <span class="text-xs text-text-muted">/</span>
            <template v-if="!isEditingTitle">
              <span
                @click="startTitleEdit"
                class="text-xs text-text-primary truncate cursor-pointer hover:text-accent transition-colors"
              >
                {{ unsavedForm.name || t('request.newRequest') }}
              </span>
            </template>
            <template v-else>
              <input
                v-model="editedTitle"
                @blur="finishTitleEdit"
                @keyup.enter="finishTitleEdit"
                @keyup.esc="isEditingTitle = false"
                class="min-w-[80px] max-w-[200px] px-1.5 py-0.5 text-xs bg-surface-deep border border-border-default rounded-sm text-text-primary focus:outline-hidden focus:border-accent"
                placeholder="Request name"
                autofocus
              />
            </template>
          </div>
          <button
            @click="handleCreateRequest"
            :disabled="!unsavedForm.name.trim() || !unsavedForm.path.trim() || !unsavedForm.collectionId"
            class="inline-flex items-center justify-center px-2 py-1 rounded-sm text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-surface-elevated transition-colors duration-150 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"/>
            </svg>
            {{ t('request.save') }}
          </button>
        </div>

        <div class="flex items-center gap-2">
          <MethodSelect
            :model-value="unsavedForm.method"
            @update:model-value="handleUnsavedMethodChange"
          />
          <input
            v-model="unsavedForm.path"
            @input="handleUnsavedPathCheck"
            :placeholder="t('request.pathPlaceholder')"
            class="flex-1 min-w-0 block rounded-r border border-border-default bg-surface-base px-2 py-1.5 text-xs focus:outline-hidden focus:border-accent text-text-primary placeholder:text-muted font-mono text-[11px] leading-none"
          />
        </div>
        <div v-if="duplicateError && isUnsavedTab" class="mt-0.5 text-xs text-error">
          {{ duplicateError }}
        </div>

        <div class="flex items-center gap-6">
          <div class="flex items-center gap-3">
            <label class="text-xs text-text-secondary w-20">{{ t('request.statusCode') }}</label>
            <input
              v-model.number="unsavedForm.statusCode"
              type="number"
              class="w-20 px-2 py-1 text-xs bg-surface-deep border border-border-default rounded-sm text-text-primary focus:outline-hidden focus:border-accent [appearance:textfield] [&::-webkit-inner-spin-button]:hidden [&::-webkit-outer-spin-button]:hidden"
            />
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs text-text-secondary">{{ t('request.delay') }}</label>
            <input
              v-model.number="unsavedForm.delayMs"
              type="number"
              class="w-20 px-2 py-1 text-xs bg-surface-deep border border-border-default rounded-sm text-text-primary focus:outline-hidden focus:border-accent [appearance:textfield] [&::-webkit-inner-spin-button]:hidden [&::-webkit-outer-spin-button]:hidden"
            />
          </div>
        </div>

        <div class="bg-surface-deep rounded-sm border border-border-default overflow-hidden">
          <div class="px-3 py-1.5 border-b border-border-default bg-surface-elevated/50 flex items-center justify-start">
            <span class="text-[10px] font-semibold text-text-secondary uppercase tracking-wide">{{ t('request.headersLabel') }}</span>
          </div>
          <div class="relative">
            <textarea
              ref="unsavedHeadersTextarea"
              v-model="unsavedForm.headers"
              class="w-full h-48 p-3 pr-20 text-xs font-mono text-text-secondary bg-transparent resize-none focus:outline-hidden"
            />
            <div
              class="absolute bottom-0 left-0 right-0 h-4 cursor-row-resize z-10"
              @mousedown.stop="startResize($event, $refs.unsavedHeadersTextarea as HTMLTextAreaElement)"
            />
            <button
              v-if="unsavedForm.headers.trim()"
              @click="unsavedForm.headers = prettifyHeaders(unsavedForm.headers)"
              class="absolute bottom-2 right-2 inline-flex items-center gap-1 px-2 py-0.5 text-[10px] text-accent hover:text-cyan-300 bg-accent/10 hover:bg-accent/20 rounded-sm transition-colors z-20"
            >
              <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16" />
              </svg>
              {{ t('request.prettify') }}
            </button>
          </div>
        </div>

        <div class="bg-surface-deep rounded-sm border border-border-default overflow-hidden">
          <div class="px-3 py-1.5 border-b border-border-default bg-surface-elevated/50 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-[10px] font-semibold text-text-secondary uppercase tracking-wide">{{ t('request.responseBody') }}</span>
              <select
                v-model="unsavedForm.bodyType"
                class="px-2 py-0.5 text-xs bg-surface-deep border border-border-default rounded-sm text-text-primary focus:outline-hidden focus:border-accent"
              >
                <option value="json">{{ t('request.json') }}</option>
                <option value="text">{{ t('request.text') }}</option>
                <option value="html">{{ t('request.html') }}</option>
                <option value="xml">{{ t('request.xml') }}</option>
              </select>
            </div>
          </div>
          <div class="relative">
            <textarea
              ref="unsavedBodyTextarea"
              v-model="unsavedForm.body"
              class="w-full h-64 p-3 pr-20 text-xs font-mono text-text-secondary bg-transparent resize-none focus:outline-hidden"
            />
            <div
              class="absolute bottom-0 left-0 right-0 h-4 cursor-row-resize z-10"
              @mousedown.stop="startResize($event, $refs.unsavedBodyTextarea as HTMLTextAreaElement)"
            />
            <button
              v-if="unsavedForm.bodyType === 'json' && unsavedForm.body.trim()"
              @click="prettifyBody(unsavedForm)"
              class="absolute bottom-2 right-2 inline-flex items-center gap-1 px-2 py-0.5 text-[10px] text-accent hover:text-cyan-300 bg-accent/10 hover:bg-accent/20 rounded-sm transition-colors z-20"
            >
              <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16" />
              </svg>
              {{ t('request.prettify') }}
            </button>
          </div>
        </div>
      </div>

      <div v-else-if="collectionsStore.examples.length === 0" class="flex items-center justify-center h-full">
        <div class="text-text-muted text-sm">{{ t('request.noExamples') }}</div>
      </div>

      <div v-else class="space-y-3">
        <div class="flex items-center gap-1">
          <div class="flex items-center gap-1 flex-1 min-w-0">
            <Listbox
              :model-value="editForm.collectionId"
              @update:model-value="editForm.collectionId = $event"
            >
              <div class="relative">
                <ListboxButton
                  class="text-xs bg-transparent border-none text-text-muted focus:outline-hidden cursor-pointer py-0.5 pr-1 flex items-center gap-0.5"
                >
                  <span class="truncate">{{ editCollectionName || '' }}</span>
                  <svg class="w-3 h-3 opacity-60 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                  </svg>
                </ListboxButton>

                <transition
                  leave-active-class="transition duration-100 ease-in"
                  leave-from-class="opacity-100"
                  leave-to-class="opacity-0"
                >
                  <ListboxOptions
                    class="absolute z-10 mt-1 bg-surface-base shadow-xl max-h-60 rounded-md border border-border-default focus:outline-hidden overflow-hidden w-max"
                  >
                    <ListboxOption
                      v-for="c in collectionsStore.collections"
                      :key="c.id"
                      :value="c.id"
                      v-slot="{ active, selected }"
                    >
                      <li
                        class="cursor-pointer select-none relative py-1.5 pl-3 pr-9 transition-colors duration-100 text-xs whitespace-nowrap"
                        :class="[
                          active ? 'bg-accent/10 text-text-primary' : 'text-text-secondary',
                          selected ? 'font-semibold' : 'font-normal'
                        ]"
                      >
                        {{ c.name }}
                        <span v-if="selected" class="absolute inset-y-0 right-0 flex items-center pr-3 text-accent">
                          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                          </svg>
                        </span>
                      </li>
                    </ListboxOption>
                  </ListboxOptions>
                </transition>
              </div>
            </Listbox>
            <span class="text-xs text-text-muted">/</span>
            <template v-if="!isEditingRequestName">
              <span
                @click="startRequestNameEdit"
                class="text-xs text-text-primary truncate cursor-pointer hover:text-accent transition-colors"
              >
                {{ activeRequestName || t('request.newRequest') }}
              </span>
            </template>
            <template v-else>
              <input
                v-model="editedRequestName"
                @blur="finishRequestNameEdit"
                @keyup.enter="finishRequestNameEdit"
                @keyup.esc="isEditingRequestName = false"
                class="min-w-[80px] max-w-[200px] px-1.5 py-0.5 text-xs bg-surface-deep border border-border-default rounded-sm text-text-primary focus:outline-hidden focus:border-accent"
                :placeholder="t('request.namePlaceholder')"
                autofocus
              />
            </template>
          </div>
          <button
            @click="handleUpdateRequest"
            class="inline-flex items-center justify-center px-2 py-1 rounded-sm text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-surface-elevated transition-colors duration-150"
          >
            <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"/>
            </svg>
            {{ t('request.save') }}
          </button>
        </div>

        <div class="flex items-center gap-2">
          <MethodSelect
            v-model="editForm.method"
          />
          <input
            v-model="editForm.path"
            @input="handleSavedPathCheck"
            :placeholder="t('request.savedPathPlaceholder')"
            class="flex-1 min-w-0 block rounded-r border border-border-default bg-surface-base px-2 py-1.5 text-xs focus:outline-hidden focus:border-accent text-text-primary placeholder-text-muted font-mono text-[11px] leading-none"
          />
        </div>
        <div v-if="duplicateError && !isUnsavedTab" class="mt-0.5 text-xs text-error">
          {{ duplicateError }}
        </div>

        <div class="flex items-center gap-6">
          <div class="flex items-center gap-3">
            <label class="text-xs text-text-secondary w-20">{{ t('request.statusCode') }}</label>
            <input
              v-model.number="editForm.statusCode"
              type="number"
              class="w-20 px-2 py-1 text-xs bg-surface-deep border border-border-default rounded-sm text-text-primary focus:outline-hidden focus:border-accent [appearance:textfield] [&::-webkit-inner-spin-button]:hidden [&::-webkit-outer-spin-button]:hidden"
            />
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs text-text-secondary">{{ t('request.delay') }}</label>
            <input
              v-model.number="editForm.delayMs"
              type="number"
              class="w-20 px-2 py-1 text-xs bg-surface-deep border border-border-default rounded-sm text-text-primary focus:outline-hidden focus:border-accent [appearance:textfield] [&::-webkit-inner-spin-button]:hidden [&::-webkit-outer-spin-button]:hidden"
            />
          </div>
        </div>

        <div class="bg-surface-deep rounded-sm border border-border-default overflow-hidden">
          <div class="px-3 py-1.5 border-b border-border-default bg-surface-elevated/50 flex items-center justify-start">
            <span class="text-[10px] font-semibold text-text-secondary uppercase tracking-wide">{{ t('request.headersLabel') }}</span>
          </div>
          <div class="relative">
            <textarea
              ref="editHeadersTextarea"
              v-model="editForm.headers"
              class="w-full h-48 p-3 pr-20 text-xs font-mono text-text-secondary bg-transparent resize-none focus:outline-hidden"
            />
            <div
              class="absolute bottom-0 left-0 right-0 h-4 cursor-row-resize z-10"
              @mousedown.stop="startResize($event, $refs.editHeadersTextarea as HTMLTextAreaElement)"
            />
            <button
              v-if="editForm.headers.trim()"
              @click="editForm.headers = prettifyHeaders(editForm.headers)"
              class="absolute bottom-2 right-2 inline-flex items-center gap-1 px-2 py-0.5 text-[10px] text-accent hover:text-cyan-300 bg-accent/10 hover:bg-accent/20 rounded-sm transition-colors z-20"
            >
              <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16" />
              </svg>
              {{ t('request.prettify') }}
            </button>
          </div>
        </div>

        <div class="bg-surface-deep rounded-sm border border-border-default overflow-hidden">
          <div class="px-3 py-1.5 border-b border-border-default bg-surface-elevated/50 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-[10px] font-semibold text-text-secondary uppercase tracking-wide">{{ t('request.responseBody') }}</span>
              <select
                v-model="editForm.bodyType"
                class="px-2 py-0.5 text-xs bg-surface-deep border border-border-default rounded-sm text-text-primary focus:outline-hidden focus:border-accent"
              >
                <option value="json">{{ t('request.json') }}</option>
                <option value="text">{{ t('request.text') }}</option>
                <option value="html">{{ t('request.html') }}</option>
                <option value="xml">{{ t('request.xml') }}</option>
              </select>
            </div>
          </div>
          <div class="relative">
            <textarea
              ref="editBodyTextarea"
              v-model="editForm.body"
              class="w-full h-64 p-3 pr-20 text-xs font-mono text-text-secondary bg-transparent resize-none focus:outline-hidden"
            />
            <div
              class="absolute bottom-0 left-0 right-0 h-4 cursor-row-resize z-10"
              @mousedown.stop="startResize($event, $refs.editBodyTextarea as HTMLTextAreaElement)"
            />
            <button
              v-if="editForm.bodyType === 'json' && editForm.body.trim()"
              @click="prettifyBody(editForm)"
              class="absolute bottom-2 right-2 inline-flex items-center gap-1 px-2 py-0.5 text-[10px] text-accent hover:text-cyan-300 bg-accent/10 hover:bg-accent/20 rounded-sm transition-colors z-20"
            >
              <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16" />
              </svg>
              {{ t('request.prettify') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
