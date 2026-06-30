import type { DashboardSnapshot, PublicSnapshot, UpstreamSettings, ClientKeyEntry } from "../types";

export async function fetchSnapshot(token: string, signal?: AbortSignal): Promise<DashboardSnapshot> {
  const response = await fetch("/api/state", {
    headers: authorizationHeaders(token),
    signal,
  });
  if (!response.ok)
    throw new Error(`HTTP ${response.status}`);
  return (await response.json()) as DashboardSnapshot;
}

export async function fetchPublicSnapshot(signal?: AbortSignal): Promise<PublicSnapshot> {
  const response = await fetch("/api/share", { signal });
  if (!response.ok)
    throw new Error(`HTTP ${response.status}`);
  return (await response.json()) as PublicSnapshot;
}

export async function saveUpstreamSettings(
  settings: UpstreamSettings,
  token: string,
): Promise<DashboardSnapshot> {
  const response = await fetch("/api/upstream", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...authorizationHeaders(token),
    },
    body: JSON.stringify(settings),
  });
  if (!response.ok)
    throw new Error(`HTTP ${response.status}`);
  return (await response.json()) as DashboardSnapshot;
}

export function formatUptime(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const days = Math.floor(total / 86400);
  const hours = Math.floor((total % 86400) / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;
  if (days > 0)
    return `${days}d ${hours}h ${minutes}m`;
  if (hours > 0)
    return `${hours}h ${minutes}m ${secs}s`;
  if (minutes > 0)
    return `${minutes}m ${secs}s`;
  return `${secs}s`;
}

export function formatRelative(unix: number | null, nowMs: number): string {
  if (!unix)
    return "—";
  const diff = Math.max(0, Math.floor(nowMs / 1000 - unix));
  if (diff < 60)
    return `${diff}s 前`;
  if (diff < 3600)
    return `${Math.floor(diff / 60)}m 前`;
  if (diff < 86400)
    return `${Math.floor(diff / 3600)}h 前`;
  return `${Math.floor(diff / 86400)}d 前`;
}

export function formatClock(unix: number): string {
  const date = new Date(unix * 1000);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

export function protocolLabel(protocol: string | null): string {
  if (!protocol)
    return "未配置";
  return protocol === "mix_space" ? "Mix-Space" : "Native";
}

function authorizationHeaders(token: string): Record<string, string> {
  const normalized = token.trim();
  return normalized ? { Authorization: `Bearer ${normalized}` } : {};
}

export async function login(username: string, password: string):Promise<{token: string}> {
  const response = await fetch("/api/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password })
  });
  if (!response.ok) throw new Error("Invalid username or password");
  return response.json();
}

export async function fetchClientKeys(token: string): Promise<ClientKeyEntry[]> {
  const response = await fetch("/api/clients/keys", {
    headers: authorizationHeaders(token)
  });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

export async function createClientKey(token: string, description: string): Promise<{api_key: string}> {
  const response = await fetch("/api/clients/keys", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...authorizationHeaders(token)
    },
    body: JSON.stringify({ description })
  });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

export async function deleteClientKey(token: string, id: number): Promise<{success: boolean}> {
  const response = await fetch(`/api/clients/keys/${id}`, {
    method: "DELETE",
    headers: authorizationHeaders(token)
  });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}
