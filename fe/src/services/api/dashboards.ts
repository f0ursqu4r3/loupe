import { api } from './client'
import type {
  Dashboard,
  Tile,
  CreateDashboardRequest,
  UpdateDashboardRequest,
  CreateTileRequest,
  UpdateTileRequest,
  UUID,
  PaginatedResponse,
  DashboardFilterParams,
} from '@/types'

export const dashboardsApi = {
  list(params?: DashboardFilterParams): Promise<PaginatedResponse<Dashboard>> {
    return api.get<PaginatedResponse<Dashboard>>('/dashboards', { params })
  },

  get(id: UUID): Promise<Dashboard> {
    return api.get<Dashboard>(`/dashboards/${id}`)
  },

  create(data: CreateDashboardRequest): Promise<Dashboard> {
    return api.post<Dashboard>('/dashboards', data)
  },

  update(id: UUID, data: UpdateDashboardRequest): Promise<Dashboard> {
    return api.put<Dashboard>(`/dashboards/${id}`, data)
  },

  delete(id: UUID): Promise<void> {
    return api.delete(`/dashboards/${id}`)
  },

  // Tiles
  listTiles(dashboardId: UUID): Promise<Tile[]> {
    return api.get<Tile[]>(`/dashboards/${dashboardId}/tiles`)
  },

  createTile(dashboardId: UUID, data: CreateTileRequest): Promise<Tile> {
    return api.post<Tile>(`/dashboards/${dashboardId}/tiles`, data)
  },

  updateTile(dashboardId: UUID, tileId: UUID, data: UpdateTileRequest): Promise<Tile> {
    return api.put<Tile>(`/dashboards/${dashboardId}/tiles/${tileId}`, data)
  },

  deleteTile(dashboardId: UUID, tileId: UUID): Promise<void> {
    return api.delete(`/dashboards/${dashboardId}/tiles/${tileId}`)
  },
}
