import type {
  OnlineStatus,
  ActivityInfo,
  MediaInfo,
  DeviceInfo,
  Stats,
  Snapshot,
} from './status-data'

export interface ClientKeyEntry {
  id: number
  description: string
  api_key: string
  created_at: number
}

/** Client kind as serialized by the Rust enum in admin endpoints. */
export type AdminClientKind = 'desktop_reporter' | 'mobile'

export interface UpstreamSettings {
  protocol: 'native' | 'mix_space'
  enable_media_reporting: boolean
  native_ws_url: string
  native_token: string
  mix_space_endpoint: string
  mix_space_method: string
  mix_space_token: string
  s3_enabled: boolean
  s3_bucket: string
  s3_region: string
  s3_access_key: string
  s3_secret_key: string
  s3_endpoint: string
  s3_custom_domain: string
  s3_key_template: string
}

export interface AccessSettings {
  accept_desktop: boolean
  accept_mobile: boolean
  activity_log_limit: number
}

export interface DataSummary {
  total_events: number
  total_messages: number
  window_info_count: number
  media_playback_count: number
  artwork_uploads: number
  upstream_errors: number
}

export interface AdminSnapshot {
  started_at: number
  bind_addr: string
  uptime_seconds: number
  config: {
    upstream_enabled: boolean
    upstream_protocol: string
    media_reporting_enabled: boolean
    s3_enabled: boolean
    native_configured: boolean
    mix_space_configured: boolean
    desktop_accepts_clients: boolean
    mobile_accepts_clients: boolean
  }
  stats: {
    total_messages: number
    window_info_count: number
    media_playback_count: number
    artwork_uploads: number
    upstream_errors: number
    native_upstream_connections: number
    last_activity_at: number | null
  }
  clients: Array<{
    id: number
    kind: AdminClientKind
    connected_at: number
    client_info: string | null
    device_id: string | null
    session_id: number
    last_window: string | null
    last_media: string | null
    messages: number
  }>
  activity: Array<{
    ts: number
    kind: string
    client: AdminClientKind | null
    client_id: number | null
    summary: string
    detail: string | null
  }>
  upstream: UpstreamSettings
  access: AccessSettings
}

// Re-export view types for components that construct Snapshots locally.
export type {
  OnlineStatus,
  ActivityInfo,
  MediaInfo,
  DeviceInfo,
  Stats,
  Snapshot,
}

const TOKEN_KEY = 'shikenmatrix_admin_token'
const DEV_API_ORIGIN = 'http://127.0.0.1:4317'

export function apiUrl(path: string): string {
  if (
    /^(?:[a-z][a-z\d+\-.]*:)?\/\//i.test(path) ||
    path.startsWith('data:') ||
    path.startsWith('blob:')
  ) {
    return path
  }

  const normalizedPath = path.startsWith('/') ? path : `/${path}`

  if (typeof window === 'undefined') {
    return normalizedPath
  }

  const isPanelDev =
    (window.location.hostname === '127.0.0.1' ||
      window.location.hostname === 'localhost') &&
    window.location.port === '4400'

  return isPanelDev ? `${DEV_API_ORIGIN}${normalizedPath}` : normalizedPath
}

export function getToken(): string | null {
  try {
    return localStorage.getItem(TOKEN_KEY)
  } catch {
    return null
  }
}

export function setToken(token: string): void {
  try {
    localStorage.setItem(TOKEN_KEY, token)
  } catch {
    /* ignore */
  }
}

export function clearToken(): void {
  try {
    localStorage.removeItem(TOKEN_KEY)
  } catch {
    /* ignore */
  }
}

function authHeaders(): HeadersInit {
  const token = getToken()
  return token ? { Authorization: `Bearer ${token}` } : {}
}

async function request<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(apiUrl(path), {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders(),
      ...(init?.headers ?? {}),
    },
  })

  if (res.status === 401) {
    clearToken()
    throw new ApiError('未登录或登录已过期', 401)
  }

  if (!res.ok) {
    let message = `请求失败 (${res.status})`
    try {
      const body = await res.json()
      if (body && typeof body.error === 'string') message = body.error
    } catch {
      /* ignore */
    }
    throw new ApiError(message, res.status)
  }

  return (await res.json()) as T
}

export class ApiError extends Error {
  status: number
  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

export const api = {
  async login(username: string, password: string): Promise<string> {
    const body = await request<{ token: string }>('/api/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    })
    setToken(body.token)
    return body.token
  },

  async logout(): Promise<void> {
    clearToken()
  },

  /** Best-effour auth check: returns true when the stored token is accepted. */
  async checkAuth(): Promise<boolean> {
    const token = getToken()
    if (!token) return false
    try {
      await request<AdminSnapshot>('/api/state', { method: 'GET' })
      return true
    } catch {
      return false
    }
  },

  async getState(): Promise<AdminSnapshot> {
    return request<AdminSnapshot>('/api/state', { method: 'GET' })
  },

  async getUpstream(): Promise<UpstreamSettings> {
    return request<UpstreamSettings>('/api/upstream', { method: 'GET' })
  },

  async saveUpstream(settings: UpstreamSettings): Promise<UpstreamSettings> {
    return request<UpstreamSettings>('/api/upstream', {
      method: 'PUT',
      body: JSON.stringify(settings),
    })
  },

  async getAccess(): Promise<AccessSettings> {
    return request<AccessSettings>('/api/access', { method: 'GET' })
  },

  async saveAccess(settings: AccessSettings): Promise<AccessSettings> {
    return request<AccessSettings>('/api/access', {
      method: 'PUT',
      body: JSON.stringify(settings),
    })
  },

  async changePassword(currentPassword: string, newPassword: string): Promise<void> {
    await request<{ success: boolean }>('/api/account/password', {
      method: 'PUT',
      body: JSON.stringify({
        current_password: currentPassword,
        new_password: newPassword,
      }),
    })
  },

  async getData(): Promise<DataSummary> {
    return request<DataSummary>('/api/data', { method: 'GET' })
  },

  async clearActivity(): Promise<void> {
    await request<{ success: boolean }>('/api/data/activity', {
      method: 'DELETE',
    })
  },

  async resetStats(): Promise<void> {
    await request<{ success: boolean }>('/api/data/stats', {
      method: 'POST',
    })
  },

  async getClientKeys(): Promise<ClientKeyEntry[]> {
    return request<ClientKeyEntry[]>('/api/clients/keys', { method: 'GET' })
  },

  async createClientKey(description: string): Promise<{ api_key: string }> {
    return request<{ api_key: string }>('/api/clients/keys', {
      method: 'POST',
      body: JSON.stringify({ description }),
    })
  },

  async deleteClientKey(id: number): Promise<void> {
    await request<{ success: boolean }>(`/api/clients/keys/${id}`, {
      method: 'DELETE',
    })
  },
}
