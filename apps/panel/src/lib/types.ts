/** Client kind as reported by the server snapshot. */
export type ClientKind = "desktop" | "mobile";

export interface OnlineStatus {
  is_online: boolean;
  screen_off: boolean | null;
  client_kind: ClientKind | string;
  device_info: string | null;
  last_activity_at: string | null;
}

export interface ActivityInfo {
  process_name: string | null;
  title: string | null;
  icon_url: string | null;
  app_id: string | null;
}

export interface MediaInfo {
  title: string | null;
  artist: string | null;
  album: string | null;
  duration: number | null;
  elapsed_time: number | null;
  playing: boolean | null;
  artwork_url: string | null;
}

export interface DeviceInfo {
  battery_level: number | null;
  battery_charging: boolean | null;
  network_wifi: boolean | null;
  network_cellular: boolean | null;
  network_vpn: boolean | null;
  latitude: number | null;
  longitude: number | null;
}

/** Payload pushed over `/api/share/{kind}/events`. */
export interface ClientSnapshot {
  status: OnlineStatus;
  activity: ActivityInfo | null;
  media: MediaInfo | null;
  device: DeviceInfo | null;
  stats: { total_messages: number };
}

/** Live transport state of one device feed. */
export type FeedState = "connecting" | "live" | "offline";

export interface ClientKeyEntry {
  id: number;
  description: string;
  api_key: string;
  created_at: number;
}

/** Client kind as serialized by the Rust enum in admin endpoints. */
export type AdminClientKind = "desktop_reporter" | "mobile";

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

export interface AccessSettings {
  accept_desktop: boolean;
  accept_mobile: boolean;
  activity_log_limit: number;
}

export interface DataSummary {
  total_events: number;
  total_messages: number;
  window_info_count: number;
  media_playback_count: number;
  artwork_uploads: number;
  upstream_errors: number;
}

export interface AdminSnapshot {
  started_at: number;
  bind_addr: string;
  uptime_seconds: number;
  config: {
    upstream_enabled: boolean;
    upstream_protocol: string;
    media_reporting_enabled: boolean;
    s3_enabled: boolean;
    native_configured: boolean;
    mix_space_configured: boolean;
    desktop_accepts_clients: boolean;
    mobile_accepts_clients: boolean;
  };
  stats: {
    total_messages: number;
    window_info_count: number;
    media_playback_count: number;
    artwork_uploads: number;
    upstream_errors: number;
    native_upstream_connections: number;
    last_activity_at: number | null;
  };
  clients: Array<{
    id: number;
    kind: AdminClientKind;
    connected_at: number;
    client_info: string | null;
    device_id: string | null;
    session_id: number;
    last_window: string | null;
    last_media: string | null;
    messages: number;
  }>;
  activity: Array<{
    ts: number;
    kind: string;
    client: AdminClientKind | null;
    client_id: number | null;
    summary: string;
    detail: string | null;
  }>;
  upstream: UpstreamSettings;
  access: AccessSettings;
}
