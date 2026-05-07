import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { logger } from '../lib/logger'
import type { MockDraft } from './types'

export interface LogEntry {
  id: string
  time: string
  method: string
  path: string
  status: number
  query?: string
  headers?: [string, string][]
  body?: string
}

// snake_case fields match Rust TempRequest struct for serde deserialization
export interface TempRequest {
  tab_id: string
  method: string
  path: string
  status_code: number
  headers: string
  body: string
  body_type: string
  delay_ms: string | null
}

function toTempRequests(drafts: MockDraft[]): TempRequest[] {
  return drafts.map(d => ({
    tab_id: d.tabId,
    method: d.state.method,
    path: d.state.path,
    status_code: d.state.statusCode,
    headers: d.state.headers,
    body: d.state.body,
    body_type: d.state.bodyType,
    delay_ms: d.state.delayMs || null,
  }))
}

export const useServerStore = defineStore('server', () => {
  const isRunning = ref(false)
  const port = ref(3210)
  const logs = ref<LogEntry[]>([])
  let unlisten: UnlistenFn | null = null

  async function setupListener() {
    if (unlisten) return
    unlisten = await listen<LogEntry>('server-log', (event) => {
      logs.value.unshift(event.payload)
      if (logs.value.length > 500) {
        logs.value = logs.value.slice(0, 500)
      }
    })
  }

  function cleanupListener() {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  async function loadStatus() {
    try {
      const status = await invoke<{ running: boolean; port: number }>('get_server_status')
      isRunning.value = status.running
      port.value = status.port
    } catch (e) {
      logger.error('Failed to get server status:', e)
    }
  }

  async function startServer(corsOrigins: string[] = ['*'], drafts: MockDraft[] = []) {
    try {
      const resultPort = await invoke<number>('start_server', {
        port: port.value,
        corsOrigins,
        drafts: toTempRequests(drafts),
      })
      isRunning.value = true
      port.value = resultPort
    } catch (e) {
      logger.error('Failed to start server:', e)
      throw e
    }
  }

  async function stopServer() {
    try {
      await invoke('stop_server')
      isRunning.value = false
    } catch (e) {
      logger.error('Failed to stop server:', e)
      throw e
    }
  }

  function clearLogs() {
    logs.value = []
  }

  return {
    isRunning,
    port,
    logs,
    loadStatus,
    startServer,
    stopServer,
    clearLogs,
    setupListener,
    cleanupListener,
  }
})
