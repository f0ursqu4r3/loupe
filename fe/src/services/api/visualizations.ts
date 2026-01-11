import { api } from './client'
import type {
  Visualization,
  CreateVisualizationRequest,
  UpdateVisualizationRequest,
  UUID,
} from '@/types'

export const visualizationsApi = {
  list(queryId?: UUID): Promise<Visualization[]> {
    return api.get<Visualization[]>('/visualizations', { params: { query_id: queryId } })
  },

  get(id: UUID): Promise<Visualization> {
    return api.get<Visualization>(`/visualizations/${id}`)
  },

  create(data: CreateVisualizationRequest): Promise<Visualization> {
    return api.post<Visualization>('/visualizations', data)
  },

  update(id: UUID, data: UpdateVisualizationRequest): Promise<Visualization> {
    return api.patch<Visualization>(`/visualizations/${id}`, data)
  },

  delete(id: UUID): Promise<void> {
    return api.delete(`/visualizations/${id}`)
  },
}
