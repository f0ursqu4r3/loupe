import { api } from './client'
import type {
  Datasource,
  CreateDatasourceRequest,
  UpdateDatasourceRequest,
  ConnectionTestResult,
  UUID,
  PaginatedResponse,
  DatasourceFilterParams,
} from '@/types'

export const datasourcesApi = {
  list(params?: DatasourceFilterParams): Promise<PaginatedResponse<Datasource>> {
    return api.get<PaginatedResponse<Datasource>>('/datasources', { params })
  },

  get(id: UUID): Promise<Datasource> {
    return api.get<Datasource>(`/datasources/${id}`)
  },

  create(data: CreateDatasourceRequest): Promise<Datasource> {
    return api.post<Datasource>('/datasources', data)
  },

  update(id: UUID, data: UpdateDatasourceRequest): Promise<Datasource> {
    return api.put<Datasource>(`/datasources/${id}`, data)
  },

  delete(id: UUID): Promise<void> {
    return api.delete(`/datasources/${id}`)
  },

  test(id: UUID): Promise<ConnectionTestResult> {
    return api.post<ConnectionTestResult>(`/datasources/${id}/test`)
  },
}
