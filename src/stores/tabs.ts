import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { Tab, MockDraft, MockDraftState } from './types'

const TABS_KEY = 'portfake-tabs'
const DRAFTS_KEY = 'portfake-drafts'

export const useTabsStore = defineStore('tabs', () => {
  const tabs = ref<Tab[]>([])
  const drafts = ref<MockDraft[]>([])
  const activeTabId = ref<string | null>(null)

  const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value) || null)

  const loadTabs = () => {
    const stored = localStorage.getItem(TABS_KEY)
    if (stored) {
      try {
        const parsed = JSON.parse(stored)
        tabs.value = parsed.tabs || []
        activeTabId.value = parsed.activeTabId || null
      } catch {
        tabs.value = []
        activeTabId.value = null
      }
    }
    const draftsStored = localStorage.getItem(DRAFTS_KEY)
    if (draftsStored) {
      try {
        drafts.value = JSON.parse(draftsStored)
      } catch {
        drafts.value = []
      }
    }
  }

  /** Create a new tab or focus existing one if requestId is already open */
  const createTab = (requestId: string, method: string, title: string) => {
    if (requestId) {
      const existing = tabs.value.find(t => t.requestId === requestId)
      if (existing) {
        activeTabId.value = existing.id
        return existing
      }
    }

    const maxPosition = tabs.value.reduce((max, t) => Math.max(max, t.position), 0)
    const tab: Tab = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      requestId,
      method,
      title,
      position: maxPosition + 1,
    }

    tabs.value.push(tab)
    activeTabId.value = tab.id
    return tab
  }

  /** Close tab and clean up associated draft, activate adjacent tab */
  const closeTab = (tabId: string) => {
    const index = tabs.value.findIndex(t => t.id === tabId)
    if (index === -1) return

    tabs.value.splice(index, 1)

    const draftIndex = drafts.value.findIndex(d => d.tabId === tabId)
    if (draftIndex !== -1) {
      drafts.value.splice(draftIndex, 1)
    }

    if (activeTabId.value === tabId) {
      if (tabs.value[index - 1]) {
        activeTabId.value = tabs.value[index - 1].id
      } else if (tabs.value[0]) {
        activeTabId.value = tabs.value[0].id
      } else {
        activeTabId.value = null
      }
    }
  }

  const setActiveTab = (tabId: string) => {
    activeTabId.value = tabId
  }

  const updateTabTitle = (tabId: string, title: string) => {
    const tab = tabs.value.find(t => t.id === tabId)
    if (tab) {
      tab.title = title
    }
  }

  const bindTabToRequest = (tabId: string, requestId: string, method: string, title: string) => {
    const tab = tabs.value.find(t => t.id === tabId)
    if (tab) {
      tab.requestId = requestId
      tab.method = method
      tab.title = title
    }
  }

  const createDraft = (tabId: string, initialState: MockDraftState): MockDraft => {
    const draft: MockDraft = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      tabId,
      state: { ...initialState },
      changesCount: 0,
    }
    drafts.value.push(draft)
    return draft
  }

  const updateDraftState = (draftId: string, state: MockDraftState) => {
    const draft = drafts.value.find(d => d.id === draftId)
    if (draft) {
      draft.state = { ...state }
    }
  }

  const markDirty = (draftId: string) => {
    const draft = drafts.value.find(d => d.id === draftId)
    if (draft) {
      draft.changesCount++
    }
  }

  const markClean = (draftId: string) => {
    const draft = drafts.value.find(d => d.id === draftId)
    if (draft) {
      draft.changesCount = 0
    }
  }

/** Auto-persist tabs and drafts to localStorage with debounce */
  let saveTimer: ReturnType<typeof setTimeout> | null = null
  watch([tabs, drafts, activeTabId], () => {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(() => {
      localStorage.setItem(TABS_KEY, JSON.stringify({ tabs: tabs.value, activeTabId: activeTabId.value }))
      localStorage.setItem(DRAFTS_KEY, JSON.stringify(drafts.value))
    }, 500)
  }, { flush: 'post', deep: true })

  return {
    tabs,
    drafts,
    activeTabId,
    activeTab,
    loadTabs,
    createTab,
    closeTab,
    setActiveTab,
    updateTabTitle,
    bindTabToRequest,
    createDraft,
    updateDraftState,
    markDirty,
    markClean,
  }
})
