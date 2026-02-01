import { api } from './client'
import type { Schedule, CreateScheduleRequest, UpdateScheduleRequest, UUID, PaginatedResponse, ScheduleFilterParams } from '@/types'

export const schedulesApi = {
  list(params?: ScheduleFilterParams): Promise<PaginatedResponse<Schedule>> {
    return api.get<PaginatedResponse<Schedule>>('/schedules', { params })
  },

  get(id: UUID): Promise<Schedule> {
    return api.get<Schedule>(`/schedules/${id}`)
  },

  create(data: CreateScheduleRequest): Promise<Schedule> {
    return api.post<Schedule>('/schedules', data)
  },

  update(id: UUID, data: UpdateScheduleRequest): Promise<Schedule> {
    return api.patch<Schedule>(`/schedules/${id}`, data)
  },

  delete(id: UUID): Promise<void> {
    return api.delete(`/schedules/${id}`)
  },

  enable(id: UUID): Promise<Schedule> {
    return api.post<Schedule>(`/schedules/${id}/enable`)
  },

  disable(id: UUID): Promise<Schedule> {
    return api.post<Schedule>(`/schedules/${id}/disable`)
  },

  trigger(id: UUID): Promise<void> {
    return api.post(`/schedules/${id}/trigger`)
  },
}
