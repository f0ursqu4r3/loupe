import { api } from './client'
import type {
  Visualization,
  CreateVisualizationRequest,
  UpdateVisualizationRequest,
  UUID,
  PaginatedResponse,
  VisualizationFilterParams,
} from '@/types'

export const visualizationsApi = {
  list(params?: VisualizationFilterParams): Promise<PaginatedResponse<Visualization>> {
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
