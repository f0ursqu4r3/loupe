import { api } from './client'
import type { Query, CreateQueryRequest, UpdateQueryRequest, Run, QueryResult, UUID } from '@/types'

export interface CreateRunRequest {
  query_id: UUID
  parameters?: Record<string, unknown>
  timeout_seconds?: number
  max_rows?: number
}

export const queriesApi = {
  list(): Promise<Query[]> {
    return api.get<Query[]>('/queries')
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
}

export const runsApi = {
  list(queryId?: UUID): Promise<Run[]> {
    return api.get<Run[]>('/runs', { params: { query_id: queryId } })
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
