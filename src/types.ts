export type ReporterProtocol = 'native' | 'mix_space'

export interface ReporterConfig {
  protocol: ReporterProtocol
  enable_media_reporting: boolean
  native: NativeReporterConfig
  mix_space: MixSpaceReporterConfig
  s3: S3ReporterConfig
}

export interface NativeReporterConfig {
  ws_url: string
  token: string
}

export interface MixSpaceReporterConfig {
  endpoint: string
  method: string
  token: string
}

export interface S3ReporterConfig {
  enabled: boolean
  bucket: string
  region: string
  access_key: string
  secret_key: string
  endpoint: string
  custom_domain: string
  key_template: string
  lifecycle_days: number
}

export type CloseBehavior = 'hide_to_tray' | 'quit'

export type AppPage = 'monitor' | 'settings' | 'about'

export interface ConnectionStatus {
  is_monitoring: boolean
  is_reporting: boolean
  is_connected: boolean
  last_error: string | null
}

export interface PermissionStatus {
  accessibility: boolean
  media: boolean
  accessibility_required: boolean
}

export interface LogEntry {
  time: string
  level: 'Info' | 'Warn' | 'Error'
  message: string
}

export interface WindowView {
  title: string
  process_name: string
  pid: number
  iconSrc?: string
}

export interface MediaView {
  title: string
  artist: string
  album: string
  duration: number
  elapsed_time: number
  playing: boolean
  artworkSrc?: string
}
