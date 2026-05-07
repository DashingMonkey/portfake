import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { logger } from '../lib/logger'
import type { Collection, Request, Example } from './types'

export const useCollectionsStore = defineStore('collections', () => {
  const collections = ref<Collection[]>([])
  const requests = ref<Request[]>([])
  const examples = ref<Example[]>([])
  const selectedCollectionId = ref<string | null>(null)
  const selectedRequestId = ref<string | null>(null)
  const expandedCollectionIds = ref<Set<string>>(new Set())
  const isLoading = ref(false)
  const loadError = ref<string | null>(null)

  function isExpanded(id: string) {
    return expandedCollectionIds.value.has(id)
  }

  const sortCollections = () => {
    collections.value.sort((a, b) => a.name.localeCompare(b.name))
  }

  async function loadCollections() {
    isLoading.value = true
    loadError.value = null
    try {
      const result = await invoke<Collection[]>('get_collections')
      collections.value = result
      sortCollections()
    } catch (e) {
      loadError.value = String(e)
      logger.error('Failed to load collections:', e)
    } finally {
      isLoading.value = false
    }
  }

  async function createCollection(name: string) {
    const result = await invoke<Collection>('create_collection', { name })
    collections.value.push(result)
    sortCollections()
    return result
  }

  async function setCollectionEnabled(id: string, enabled: boolean) {
    await invoke('set_collection_enabled', { id, enabled })
    const index = collections.value.findIndex(c => c.id === id)
    if (index !== -1) {
      collections.value[index] = { ...collections.value[index], enabled }
    }
    if (!enabled) {
      requests.value = requests.value.map(r =>
        r.collection_id === id ? { ...r, enabled: false } : r
      )
    }
  }

  async function setRequestEnabled(id: string, enabled: boolean) {
    await invoke('set_request_enabled', { requestId: id, enabled })
    const index = requests.value.findIndex(r => r.id === id)
    if (index !== -1) {
      requests.value[index] = { ...requests.value[index], enabled }
    }
  }

  async function loadRequests(collectionId: string) {
    try {
      const result = await invoke<Request[]>('get_requests', { collectionId })
      // Replace existing requests for this collection to avoid duplicates
      requests.value = requests.value.filter(r => r.collection_id !== collectionId)
      requests.value.push(...result)
      // Sort requests by name (backend already sorts, but ensure order is maintained)
      requests.value.sort((a, b) => a.name.localeCompare(b.name))
    } catch (e) {
      logger.error('Failed to load requests:', e)
    }
  }

  async function createRequest(collectionId: string, name: string, method: string, path: string) {
    const result = await invoke<Request>('create_request', {
      collectionId,
      name,
      method,
      path,
    })
    if (expandedCollectionIds.value.has(collectionId)) {
      requests.value.push(result)
        requests.value.sort((a, b) => a.name.localeCompare(b.name))
    }
    return result
  }

  async function loadExamples(requestId: string) {
    try {
      const result = await invoke<Example[]>('get_examples', { requestId })
      examples.value = result
    } catch (e) {
      logger.error('Failed to load examples:', e)
    }
  }

  async function deleteCollection(id: string) {
    await invoke('delete_collection', { id })
    if (selectedCollectionId.value === id) {
      selectedCollectionId.value = null
    }
    requests.value = requests.value.filter(r => r.collection_id !== id)
    expandedCollectionIds.value.delete(id)
    await loadCollections()
  }

  async function renameCollection(id: string, name: string) {
    const result = await invoke<Collection>('rename_collection', { id, name })
    const index = collections.value.findIndex(c => c.id === id)
    if (index !== -1) {
      collections.value[index] = { ...collections.value[index], name }
      sortCollections()
    }
    return result
  }

  async function deleteRequest(id: string) {
    await invoke('delete_request', { requestId: id })
    requests.value = requests.value.filter(r => r.id !== id)
  }

  async function toggleExpand(collectionId: string) {
    if (expandedCollectionIds.value.has(collectionId)) {
      expandedCollectionIds.value.delete(collectionId)
      expandedCollectionIds.value = new Set(expandedCollectionIds.value)
      if (selectedCollectionId.value === collectionId) {
        selectedCollectionId.value = null
      }
    } else {
      expandedCollectionIds.value.add(collectionId)
      expandedCollectionIds.value = new Set(expandedCollectionIds.value)
      selectedCollectionId.value = collectionId
      await loadRequests(collectionId)
    }
  }

  return {
    collections,
    requests,
    examples,
    selectedCollectionId,
    selectedRequestId,
    expandedCollectionIds,
    isLoading,
    loadError,
    isExpanded,
    loadCollections,
    createCollection,
    setCollectionEnabled,
    setRequestEnabled,
    loadRequests,
    createRequest,
    loadExamples,
    deleteCollection,
    renameCollection,
    deleteRequest,
    toggleExpand,
  }
})
