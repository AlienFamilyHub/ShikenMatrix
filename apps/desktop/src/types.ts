export interface ReporterConfig {
  server: ServerReporterConfig;
}

export interface ServerReporterConfig {
  ws_url: string;
}

export type CloseBehavior = "hide_to_tray" | "quit";

export type AppPage = "monitor" | "settings" | "about";

export interface ConnectionStatus {
  is_monitoring: boolean;
  is_reporting: boolean;
  is_connected: boolean;
  last_error: string | null;
}

export interface PermissionStatus {
  accessibility: boolean;
  media: boolean;
  accessibility_required: boolean;
}

export interface LogEntry {
  time: string;
  level: "Info" | "Warn" | "Error";
  message: string;
}

export interface WindowView {
  title: string;
  process_name: string;
  pid: number;
  iconSrc?: string;
}

export interface MediaView {
  title: string;
  artist: string;
  album: string;
  duration: number;
  elapsed_time: number;
  playing: boolean;
  artworkSrc?: string;
}
