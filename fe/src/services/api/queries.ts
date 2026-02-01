import { api } from './client'
import type {
  Query,
  CreateQueryRequest,
  UpdateQueryRequest,
  Run,
  QueryResult,
  UUID,
  QueryExport,
  ImportQueriesRequest,
  ImportQueriesResponse,
  PaginatedResponse,
  PaginationParams,
} from '@/types'

export interface CreateRunRequest {
  query_id: UUID
  parameters?: Record<string, unknown>
  timeout_seconds?: number
  max_rows?: number
}

export const queriesApi = {
  list(params?: PaginationParams): Promise<PaginatedResponse<Query>> {
    return api.get<PaginatedResponse<Query>>('/queries', { params })
  },

  get(id: UUID): Promise<Query> {
    return api.get<Query>(`/queries/${id}`)
  },

  create(data: CreateQueryRequest): Promise<Query> {
    return api.post<Query>('/queries', data)
  },

  update(id: UUID, data: UpdateQueryRequest): Promise<Query> {
    return api.put<Query>(`/queries/${id}`, data)
  },

  delete(id: UUID): Promise<void> {
    return api.delete(`/queries/${id}`)
  },

  export(): Promise<QueryExport[]> {
    return api.get<QueryExport[]>('/queries/export')
  },

  import(data: ImportQueriesRequest): Promise<ImportQueriesResponse> {
    return api.post<ImportQueriesResponse>('/queries/import', data)
  },
}

export const runsApi = {
  list(queryId?: UUID, paginationParams?: PaginationParams): Promise<PaginatedResponse<Run>> {
    const params = { ...paginationParams, query_id: queryId }
    return api.get<PaginatedResponse<Run>>('/runs', { params })
  },

  get(id: UUID): Promise<Run> {
    return api.get<Run>(`/runs/${id}`)
  },

  getResult(id: UUID): Promise<QueryResult> {
    return api.get<QueryResult>(`/runs/${id}/result`)
  },

  create(data: CreateRunRequest): Promise<Run> {
    return api.post<Run>('/runs', data)
  },

  cancel(id: UUID): Promise<Run> {
    return api.post<Run>(`/runs/${id}/cancel`)
  },
}
