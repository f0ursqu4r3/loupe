export interface PaginationParams {
  limit?: number
  offset?: number
  [key: string]: string | number | boolean | undefined
}
