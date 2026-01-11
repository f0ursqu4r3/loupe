import { api } from './client'
import type {
  Query,
  CreateQueryRequest,
  UpdateQueryRequest,
  ExecuteQueryRequest,
  Run,
  QueryResult,
  UUID,
} from '@/types'

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
    return api.patch<Query>(`/queries/${id}`, data)
  },

  delete(id: UUID): Promise<void> {
    return api.delete(`/queries/${id}`)
  },

  execute(id: UUID, data?: ExecuteQueryRequest): Promise<Run> {
    return api.post<Run>(`/queries/${id}/execute`, data)
  },

  preview(id: UUID, data?: ExecuteQueryRequest): Promise<QueryResult> {
    return api.post<QueryResult>(`/queries/${id}/preview`, data)
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

  cancel(id: UUID): Promise<Run> {
    return api.post<Run>(`/runs/${id}/cancel`)
  },
}
