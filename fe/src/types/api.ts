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
  tags: string[]
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
  tags?: string[]
  timeout_seconds?: number
  max_rows?: number
}

export interface UpdateQueryRequest {
  name?: string
  description?: string
  sql?: string
  parameters?: ParamDef[]
  tags?: string[]
  timeout_seconds?: number
  max_rows?: number
}

// Query import/export
export interface QueryExport {
  name: string
  description?: string
  sql: string
  parameters: ParamDef[]
  timeout_seconds: number
  max_rows: number
  tags: string[]
  datasource_name?: string
}

export interface ImportQueriesRequest {
  queries: QueryExport[]
  datasource_id: UUID
  skip_duplicates?: boolean
}

export interface ImportQueriesResponse {
  imported: number
  skipped: number
  skipped_names: string[]
}

// ===== Run =====
export type RunStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'timeout'

export interface Run {
  id: UUID
  query_id: UUID
  status: RunStatus
  started_at?: string
  completed_at?: string
  error_message?: string
  created_at: string
}

export interface ExecuteQueryRequest {
  parameters?: Record<string, unknown>
  timeout_seconds?: number
  max_rows?: number
}

export interface QueryResult {
  run_id?: string
  columns: ColumnInfo[]
  rows: unknown[][]
  row_count: number
  execution_time_ms: number
  truncated?: boolean
}

export interface ColumnInfo {
  name: string
  data_type: string
}

// ===== Visualization =====
export type ChartType = 'table' | 'line' | 'bar' | 'pie' | 'single_stat'

export interface Visualization extends Timestamps {
  id: UUID
  org_id: UUID
  query_id: UUID
  name: string
  chart_type: ChartType
  config: VisualizationConfig
  tags: string[]
  created_by: UUID
}

export interface VisualizationConfig {
  // Display title/label
  label?: string

  // Table config
  columns?: TableColumnConfig[]

  // Chart common
  x_axis?: string
  y_axis?: string
  series_column?: string // Column to split data into multiple series (e.g., provider)

  // Line/Bar specific
  stacked?: boolean
  show_legend?: boolean
  show_grid?: boolean

  // Single stat
  value_column?: string
  prefix?: string
  suffix?: string
  thresholds?: Threshold[]

  // Pie chart
  label_column?: string
  donut?: boolean

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
  tags?: string[]
}

export interface UpdateVisualizationRequest {
  query_id?: UUID
  name?: string
  chart_type?: ChartType
  config?: VisualizationConfig
  tags?: string[]
}

// ===== Dashboard =====
export interface Dashboard extends Timestamps {
  id: UUID
  org_id: UUID
  name: string
  description?: string
  parameters: DashboardParameter[]
  tags: string[]
  tiles: Tile[]
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
  tags?: string[]
}

export interface UpdateDashboardRequest {
  name?: string
  description?: string
  parameters?: DashboardParameter[]
  tags?: string[]
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
  tags: string[]
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
  tags?: string[]
  enabled?: boolean
}

export interface UpdateScheduleRequest {
  name?: string
  cron_expression?: string
  parameters?: Record<string, unknown>
  tags?: string[]
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
