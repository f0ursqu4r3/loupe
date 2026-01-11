import { api } from './client'
import type {
  Dashboard,
  Tile,
  CreateDashboardRequest,
  UpdateDashboardRequest,
  CreateTileRequest,
  UpdateTileRequest,
  UUID,
} from '@/types'

export const dashboardsApi = {
  list(): Promise<Dashboard[]> {
    return api.get<Dashboard[]>('/dashboards')
  },

  get(id: UUID): Promise<Dashboard> {
    return api.get<Dashboard>(`/dashboards/${id}`)
  },

  create(data: CreateDashboardRequest): Promise<Dashboard> {
    return api.post<Dashboard>('/dashboards', data)
  },

  update(id: UUID, data: UpdateDashboardRequest): Promise<Dashboard> {
    return api.patch<Dashboard>(`/dashboards/${id}`, data)
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
    return api.patch<Tile>(`/dashboards/${dashboardId}/tiles/${tileId}`, data)
  },

  deleteTile(dashboardId: UUID, tileId: UUID): Promise<void> {
    return api.delete(`/dashboards/${dashboardId}/tiles/${tileId}`)
  },
}
