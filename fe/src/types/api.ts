// ===== Common Types =====
export type UUID = string

export interface Timestamps {
  created_at: string
  updated_at: string
}

// ===== User & Auth =====
export type UserRole = 'admin' | 'editor' | 'viewer'

export interface User {
  id: UUID
  org_id: UUID
  email: string
  name: string
  role: UserRole
  created_at: string
  updated_at?: string
}

export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  email: string
  password: string
  name: string
  org_name?: string
}

export interface AuthResponse {
  user: User
  token: string
}

// ===== Datasource =====
export type DatasourceType = 'postgres'

export interface Datasource extends Timestamps {
  id: UUID
  org_id: UUID
  name: string
  ds_type: DatasourceType
  created_by: UUID
}

export interface CreateDatasourceRequest {
  name: string
  ds_type?: DatasourceType
  connection_string: string
}

export interface UpdateDatasourceRequest {
  name?: string
  connection_string?: string
}

export interface ConnectionTestResult {
  success: boolean
  message: string
  latency_ms?: number
}

// ===== Query =====
export type ParamType = 'string' | 'number' | 'boolean' | 'date' | 'datetime'

export interface ParamDef {
  name: string
  param_type: ParamType
  default?: unknown
  required: boolean
}

export interface Query extends Timestamps {
  id: UUID
  org_id: UUID
  datasource_id: UUID
  name: string
  description?: string
  sql: string
  parameters: ParamDef[]
  timeout_seconds: number
  max_rows: number
  created_by: UUID
}

export interface CreateQueryRequest {
  datasource_id: UUID
  name: string
  description?: string
  sql: string
  parameters?: ParamDef[]
  timeout_seconds?: number
  max_rows?: number
}

export interface UpdateQueryRequest {
  name?: string
  description?: string
  sql?: string
  parameters?: ParamDef[]
  timeout_seconds?: number
  max_rows?: number
}

// ===== Run =====
export type RunStatus = 'pending' | 'running' | 'success' | 'error' | 'timeout' | 'cancelled'

export interface Run extends Timestamps {
  id: UUID
  org_id: UUID
  query_id: UUID
  status: RunStatus
  parameters: Record<string, unknown>
  started_at?: string
  finished_at?: string
  row_count?: number
  error_message?: string
  triggered_by: UUID
}

export interface ExecuteQueryRequest {
  parameters?: Record<string, unknown>
  timeout_seconds?: number
  max_rows?: number
}

export interface QueryResult {
  columns: ColumnInfo[]
  rows: Record<string, unknown>[]
  row_count: number
  execution_time_ms: number
  truncated: boolean
}

export interface ColumnInfo {
  name: string
  type: string
}

// ===== Visualization =====
export type ChartType = 'table' | 'line' | 'bar' | 'single_stat'

export interface Visualization extends Timestamps {
  id: UUID
  org_id: UUID
  query_id: UUID
  name: string
  chart_type: ChartType
  config: VisualizationConfig
  created_by: UUID
}

export interface VisualizationConfig {
  // Table config
  columns?: TableColumnConfig[]

  // Chart common
  x_axis?: string
  y_axis?: string | string[]
  group_by?: string

  // Line/Bar specific
  stacked?: boolean
  show_legend?: boolean
  show_grid?: boolean

  // Single stat
  value_column?: string
  prefix?: string
  suffix?: string
  thresholds?: Threshold[]

  // Colors
  colors?: string[]
}

export interface TableColumnConfig {
  key: string
  label?: string
  width?: number
  align?: 'left' | 'center' | 'right'
  format?: string
}

export interface Threshold {
  value: number
  color: string
}

export interface CreateVisualizationRequest {
  query_id: UUID
  name: string
  chart_type: ChartType
  config?: VisualizationConfig
}

export interface UpdateVisualizationRequest {
  name?: string
  chart_type?: ChartType
  config?: VisualizationConfig
}

// ===== Dashboard =====
export interface Dashboard extends Timestamps {
  id: UUID
  org_id: UUID
  name: string
  description?: string
  parameters: DashboardParameter[]
  created_by: UUID
}

export interface DashboardParameter {
  name: string
  param_type: ParamType
  default?: unknown
  label?: string
}

export interface Tile extends Timestamps {
  id: UUID
  dashboard_id: UUID
  visualization_id: UUID
  title?: string
  pos_x: number
  pos_y: number
  width: number
  height: number
  parameter_bindings: Record<string, string>
}

export interface CreateDashboardRequest {
  name: string
  description?: string
  parameters?: DashboardParameter[]
}

export interface UpdateDashboardRequest {
  name?: string
  description?: string
  parameters?: DashboardParameter[]
}

export interface CreateTileRequest {
  visualization_id: UUID
  title?: string
  pos_x?: number
  pos_y?: number
  width?: number
  height?: number
  parameter_bindings?: Record<string, string>
}

export interface UpdateTileRequest {
  title?: string
  pos_x?: number
  pos_y?: number
  width?: number
  height?: number
  parameter_bindings?: Record<string, string>
}

// ===== Schedule =====
export interface Schedule extends Timestamps {
  id: UUID
  org_id: UUID
  query_id: UUID
  name: string
  cron_expression: string
  parameters: Record<string, unknown>
  enabled: boolean
  last_run_at?: string
  next_run_at?: string
  created_by: UUID
}

export interface CreateScheduleRequest {
  query_id: UUID
  name: string
  cron_expression: string
  parameters?: Record<string, unknown>
  enabled?: boolean
}

export interface UpdateScheduleRequest {
  name?: string
  cron_expression?: string
  parameters?: Record<string, unknown>
  enabled?: boolean
}

// ===== API Response Wrappers =====
export interface ApiError {
  error: string
  message: string
  details?: unknown
}

export interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}
