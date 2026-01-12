const STORAGE_KEY = 'loupe-last-dashboard-id'

export function getLastDashboardId(): string | null {
  if (typeof window === 'undefined') return null
  return localStorage.getItem(STORAGE_KEY)
}

export function setLastDashboardId(id: string) {
  if (typeof window === 'undefined') return
  localStorage.setItem(STORAGE_KEY, id)
}

export function clearLastDashboardId() {
  if (typeof window === 'undefined') return
  localStorage.removeItem(STORAGE_KEY)
}
