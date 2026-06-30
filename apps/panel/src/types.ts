export type ClientKind = "desktop_reporter" | "mobile";

export interface ClientEntry {
  id: number;
  kind: ClientKind;
  connected_at: number;
  client_info: string | null;
  last_window: string | null;
  last_media: string | null;
  messages: number;
}

export type ActivityKind
  = | "window_info"
    | "media_playback"
    | "artwork_upload"
    | "client_connect"
    | "client_disconnect"
    | "client_rejected"
    | "config_update"
    | "upstream_error";

export interface ActivityEntry {
  ts: number;
  kind: ActivityKind;
  client: ClientKind | null;
  client_id: number | null;
  summary: string;
  detail: string | null;
}

export interface ConfigSnapshot {
  upstream_enabled: boolean;
  upstream_protocol: string;
  media_reporting_enabled: boolean;
  s3_enabled: boolean;
  native_configured: boolean;
  mix_space_configured: boolean;
  desktop_accepts_clients: boolean;
  mobile_accepts_clients: boolean;
}

export interface UpstreamSettings {
  protocol: "native" | "mix_space";
  enable_media_reporting: boolean;
  native_ws_url: string;
  native_token: string;
  mix_space_endpoint: string;
  mix_space_method: string;
  mix_space_token: string;
  s3_enabled: boolean;
  s3_bucket: string;
  s3_region: string;
  s3_access_key: string;
  s3_secret_key: string;
  s3_endpoint: string;
  s3_custom_domain: string;
  s3_key_template: string;
}

export interface StatsSnapshot {
  total_messages: number;
  window_info_count: number;
  media_playback_count: number;
  artwork_uploads: number;
  upstream_errors: number;
  native_upstream_connections: number;
  last_activity_at: number | null;
}

export interface DashboardSnapshot {
  started_at: number;
  bind_addr: string;
  uptime_seconds: number;
  config: ConfigSnapshot;
  stats: StatsSnapshot;
  clients: ClientEntry[];
  activity: ActivityEntry[];
  upstream: UpstreamSettings;
}

export interface PublicSnapshot {
  current_window: string | null;
  current_media: string | null;
  last_activity_at: number | null;
}

export interface ClientKeyEntry {
  id: number;
  description: string;
  api_key: string;
  created_at: number;
}
