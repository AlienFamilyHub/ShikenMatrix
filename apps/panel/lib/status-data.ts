export type ClientKind = 'desktop' | 'mobile'

export interface OnlineStatus {
  is_online?: boolean | null
  client_kind?: ClientKind | null
  device_info?: string | null
  last_activity_at?: string | null
}

export interface ActivityInfo {
  process_name?: string | null
  title?: string | null
  icon_url?: string | null
  app_id?: string | null
}

export interface MediaInfo {
  title?: string | null
  artist?: string | null
  album?: string | null
  duration?: number | null
  elapsed_time?: number | null
  playing?: boolean | null
  artwork_url?: string | null
}

export interface DeviceInfo {
  battery_level?: number | null
  battery_charging?: boolean | null
  network_wifi?: boolean | null
  network_cellular?: boolean | null
  network_vpn?: boolean | null
  latitude?: number | null
  longitude?: number | null
}

export interface Stats {
  total_messages?: number | null
}

export interface Snapshot {
  status?: OnlineStatus | null
  activity?: ActivityInfo | null
  media?: MediaInfo | null
  device?: DeviceInfo | null
  stats?: Stats | null
}

/** A value counts as present when it is not null/undefined and not an empty string. */
export function present(value: unknown): boolean {
  if (value === null || value === undefined) return false
  if (typeof value === 'string' && value.trim() === '') return false
  return true
}

/** True when at least one of the provided values is present. */
export function anyPresent(...values: unknown[]): boolean {
  return values.some((v) => present(v))
}

/** Format an ISO timestamp into a friendly relative label, e.g. "3 分钟前". */
export function formatRelativeTime(iso?: string | null): string | null {
  if (!present(iso)) return null
  const then = new Date(iso as string).getTime()
  if (Number.isNaN(then)) return null
  const diffMs = Date.now() - then
  const sec = Math.round(diffMs / 1000)
  if (sec < 0) return '刚刚'
  if (sec < 60) return '刚刚'
  const min = Math.round(sec / 60)
  if (min < 60) return `${min} 分钟前`
  const hr = Math.round(min / 60)
  if (hr < 24) return `${hr} 小时前`
  const day = Math.round(hr / 24)
  if (day < 30) return `${day} 天前`
  return new Date(iso as string).toLocaleDateString('zh-CN')
}

/** Format an ISO timestamp into an absolute local label. */
export function formatAbsoluteTime(iso?: string | null): string | null {
  if (!present(iso)) return null
  const d = new Date(iso as string)
  if (Number.isNaN(d.getTime())) return null
  return d.toLocaleString('zh-CN', {
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/** Format seconds into m:ss (or h:mm:ss). */
export function formatDuration(seconds?: number | null): string | null {
  if (!present(seconds) || typeof seconds !== 'number' || seconds < 0) {
    return null
  }
  const total = Math.floor(seconds)
  const h = Math.floor(total / 3600)
  const m = Math.floor((total % 3600) / 60)
  const s = total % 60
  const pad = (n: number) => n.toString().padStart(2, '0')
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`
}

export function clientKindLabel(kind?: ClientKind | null): string | null {
  if (kind === 'desktop') return 'Desktop'
  if (kind === 'mobile') return 'Android'
  return null
}
