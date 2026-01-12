const DATE_FORMATTER = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' })
const DATE_TIME_FORMATTER = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
})

function parseDate(value: unknown): Date | null {
  if (value instanceof Date) {
    return Number.isNaN(value.getTime()) ? null : value
  }

  if (typeof value === 'number') {
    const date = new Date(value)
    return Number.isNaN(date.getTime()) ? null : date
  }

  if (typeof value === 'string') {
    const trimmed = value.trim()
    if (!trimmed) return null
    const date = new Date(trimmed)
    return Number.isNaN(date.getTime()) ? null : date
  }

  return null
}

export function isDateType(dataType?: string): boolean {
  return (dataType || '').toUpperCase() === 'DATE'
}

export function isDateTimeType(dataType?: string): boolean {
  const upper = (dataType || '').toUpperCase()
  return upper === 'TIMESTAMPTZ' || upper.startsWith('TIMESTAMP')
}

export function formatDate(value: unknown): string {
  const parsed = parseDate(value)
  if (!parsed) return String(value)
  return DATE_FORMATTER.format(parsed)
}

export function formatDateTime(value: unknown): string {
  const parsed = parseDate(value)
  if (!parsed) return String(value)
  return DATE_TIME_FORMATTER.format(parsed)
}

export function formatDateLike(value: unknown, dataType?: string): string | null {
  if (isDateType(dataType)) {
    return formatDate(value)
  }
  if (isDateTimeType(dataType)) {
    return formatDateTime(value)
  }
  return null
}
