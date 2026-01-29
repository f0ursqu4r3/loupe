import type { UUID, QueryResult, Datasource, VisualizationConfig, ChartType } from './api'

// ===== Canvas Node Types =====
export type CanvasNodeType = 'query' | 'note'

export type CanvasNodeStatus = 'idle' | 'running' | 'ok' | 'error' | 'warn'

export interface CanvasNodeMeta {
  // Common
  viz?: ChartType
  vizConfig?: VisualizationConfig

  // Query node specific
  datasourceId?: UUID
  sql?: string
  status?: CanvasNodeStatus
  lastRun?: string
  rows?: number
  runtime?: string
  cached?: boolean
  queryId?: UUID // Linked backend query (created on first run)
  lastRunId?: UUID // Last successful run ID (for loading results on-demand)
  result?: QueryResult | null // Transient - not persisted to localStorage
  error?: string | null

  // Note node specific
  text?: string
}

export interface CanvasNode {
  id: string
  type: CanvasNodeType
  title: string
  x: number
  y: number
  w: number
  h: number
  meta: CanvasNodeMeta
}

// ===== Canvas Edge Types =====
export type EdgeRelationship =
  | 'motivates'
  | 'explains'
  | 'contradicts'
  | 'supports'
  | 'derives_from'
  | 'questions'

export interface CanvasEdge {
  id: string
  from: string
  to: string
  label: EdgeRelationship
}

// ===== Time Range =====
export type TimePreset =
  | 'now'
  | '1h'
  | '3h'
  | '6h'
  | '12h'
  | '24h'
  | '7d'
  | '30d'
  | '90d'
  | 'custom'

export interface TimeRange {
  preset: TimePreset
  // Offset from "now" in milliseconds (0 = now, positive = past)
  offset: number
  // For custom ranges
  start?: Date
  end?: Date
}

// ===== Canvas State =====
export interface CanvasState {
  id: string
  name: string
  nodes: CanvasNode[]
  edges: CanvasEdge[]
  timeRange: TimeRange
  live: boolean
  createdAt: string
  updatedAt: string
}

// ===== Persistence =====
export interface CanvasStorage {
  canvases: CanvasState[]
  activeCanvasId: string | null
}

// ===== Helpers =====
export function createDefaultNode(
  type: CanvasNodeType,
  position: { x: number; y: number },
  datasource?: Datasource,
): CanvasNode {
  const id = `${type[0]}-${Math.random().toString(16).slice(2, 8)}`

  if (type === 'query') {
    return {
      id,
      type: 'query',
      title: 'New Query',
      x: position.x,
      y: position.y,
      w: 280,
      h: 160,
      meta: {
        datasourceId: datasource?.id,
        sql: 'SELECT 1;',
        status: 'idle',
        lastRun: 'never',
        rows: 0,
        runtime: '-',
        cached: false,
        viz: 'table',
        vizConfig: {},
        result: null,
        error: null,
      },
    }
  }

  return {
    id,
    type: 'note',
    title: 'Note',
    x: position.x,
    y: position.y,
    w: 280,
    h: 160,
    meta: {
      text: '',
    },
  }
}

export function createDefaultEdge(from: string, to: string): CanvasEdge {
  return {
    id: `e-${Math.random().toString(16).slice(2, 8)}`,
    from,
    to,
    label: 'motivates', // Default relationship
  }
}

// Default window sizes for each preset (in milliseconds)
const PRESET_WINDOWS: Record<TimePreset, number> = {
  now: 0,
  '1h': 60 * 60 * 1000,
  '3h': 3 * 60 * 60 * 1000,
  '6h': 6 * 60 * 60 * 1000,
  '12h': 12 * 60 * 60 * 1000,
  '24h': 24 * 60 * 60 * 1000,
  '7d': 7 * 24 * 60 * 60 * 1000,
  '30d': 30 * 24 * 60 * 60 * 1000,
  '90d': 90 * 24 * 60 * 60 * 1000,
  custom: 7 * 24 * 60 * 60 * 1000, // Default to 7 days for custom
}

export function timePresetToDateRange(
  preset: TimePreset,
  offsetMs: number = 0,
): { start: Date; end: Date } {
  const now = new Date()
  // End date is "now" shifted back by the offset
  const end = new Date(now.getTime() - offsetMs)

  // Get window size based on preset
  const windowMs = PRESET_WINDOWS[preset] ?? PRESET_WINDOWS['7d']

  // Start date is end date minus the window
  const start = new Date(end.getTime() - windowMs)

  return { start, end }
}
