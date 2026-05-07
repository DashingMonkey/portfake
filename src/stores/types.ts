export interface Collection {
  id: string
  name: string
  description: string | null
  source_workspace: string
  source_collection_id: string | null
  created_at: string
  enabled: boolean
}

export interface Request {
  id: string
  collection_id: string
  name: string
  method: string
  path: string
  description: string | null
  headers: string
  position: number
  enabled: boolean
}

export interface CreateRequestParams {
  collectionId: string
  name: string
  method: string
  path: string
  exampleStatusCode?: number
  exampleHeaders?: string
  exampleBody?: string
  exampleBodyType?: string
  exampleDelayMs?: number | null
}

export interface Example {
  id: string
  request_id: string
  name: string
  is_default: boolean
  status_code: number
  headers: string
  body: string
  body_type: string
  delay_ms: string | null
  match_rules: string
  order_index: number
}

export interface Tab {
  id: string
  requestId: string
  method: string
  title: string
  position: number
}

export interface MockDraft {
  id: string
  tabId: string
  state: MockDraftState
  changesCount: number
}

export interface MockDraftState {
  collectionId: string | null
  name: string
  method: string
  path: string
  statusCode: number
  headers: string
  body: string
  bodyType: string
  delayMs: string
}

export const DEFAULT_MOCK_DRAFT_STATE: MockDraftState = {
  collectionId: null,
  name: '',
  method: 'GET',
  path: '',
  statusCode: 200,
  headers: '[\n  {\n    "key": "Content-Type",\n    "value": "application/json",\n    "enabled": true\n  }\n]',
  body: '{\n  "success": true\n}',
  bodyType: 'json',
  delayMs: '',
}

export type Theme = 'light' | 'dark'
