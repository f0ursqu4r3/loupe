import { api } from './client'
import type {
  Visualization,
  CreateVisualizationRequest,
  UpdateVisualizationRequest,
  UUID,
  PaginatedResponse,
  PaginationParams,
} from '@/types'

export const visualizationsApi = {
  list(queryId?: UUID, paginationParams?: PaginationParams): Promise<PaginatedResponse<Visualization>> {
    const params = { ...paginationParams, query_id: queryId }
    return api.get<PaginatedResponse<Visualization>>('/visualizations', { params })
  },

  get(id: UUID): Promise<Visualization> {
    return api.get<Visualization>(`/visualizations/${id}`)
  },

  create(data: CreateVisualizationRequest): Promise<Visualization> {
    return api.post<Visualization>('/visualizations', data)
  },

  update(id: UUID, data: UpdateVisualizationRequest): Promise<Visualization> {
    return api.put<Visualization>(`/visualizations/${id}`, data)
  },

  delete(id: UUID): Promise<void> {
    return api.delete(`/visualizations/${id}`)
  },
}
