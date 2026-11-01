import type {
  AccessSettings,
  AdminSnapshot,
  ClientKeyEntry,
  DataSummary,
  UpstreamSettings,
} from "./types";

const TOKEN_KEY = "shikenmatrix.admin.token";

export class ApiError extends Error {
  readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export function getToken(): string | null {
  try {
    return localStorage.getItem(TOKEN_KEY);
  } catch {
    return null;
  }
}

export function setToken(token: string): void {
  try {
    localStorage.setItem(TOKEN_KEY, token);
  } catch {
    /* storage can be unavailable in private mode; the session still works */
  }
}

export function clearToken(): void {
  try {
    localStorage.removeItem(TOKEN_KEY);
  } catch {
    /* ignore */
  }
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const token = getToken();
  const response = await fetch(path, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(token
        ? { Authorization: `Bearer ${token}` }
        : {}),
      ...(init.headers ?? {}),
    },
  });

  if (response.status === 401) {
    clearToken();
    window.dispatchEvent(new Event("shikenmatrix:unauthorized"));
    throw new ApiError("登录已过期，请重新登录", 401);
  }

  if (!response.ok) {
    let message = `请求失败 (${response.status})`;
    try {
      const body = (await response.json()) as { error?: unknown };
      if (typeof body.error === "string")
        message = body.error;
    } catch {
      /* non-JSON error bodies keep the status message */
    }
    throw new ApiError(message, response.status);
  }

  return (await response.json()) as T;
}

function send<T>(path: string, method: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method,
    body: body === undefined
      ? undefined
      : JSON.stringify(body),
  });
}

export const api = {
  async login(username: string, password: string): Promise<void> {
    const response = await send<{ token: string }>("/api/login", "POST", { username, password });
    setToken(response.token);
  },

  getState(): Promise<AdminSnapshot> {
    return request<AdminSnapshot>("/api/state");
  },

  getUpstream(): Promise<UpstreamSettings> {
    return request<UpstreamSettings>("/api/upstream");
  },

  saveUpstream(settings: UpstreamSettings): Promise<UpstreamSettings> {
    return send<UpstreamSettings>("/api/upstream", "PUT", settings);
  },

  getAccess(): Promise<AccessSettings> {
    return request<AccessSettings>("/api/access");
  },

  saveAccess(settings: AccessSettings): Promise<AccessSettings> {
    return send<AccessSettings>("/api/access", "PUT", settings);
  },

  async changePassword(currentPassword: string, newPassword: string): Promise<void> {
    await send<{ success: boolean }>("/api/account/password", "PUT", {
      current_password: currentPassword,
      new_password: newPassword,
    });
  },

  getData(): Promise<DataSummary> {
    return request<DataSummary>("/api/data");
  },

  async clearActivity(): Promise<void> {
    await send<{ success: boolean }>("/api/data/activity", "DELETE");
  },

  async resetStats(): Promise<void> {
    await send<{ success: boolean }>("/api/data/stats", "POST");
  },

  getClientKeys(): Promise<ClientKeyEntry[]> {
    return request<ClientKeyEntry[]>("/api/clients/keys");
  },

  createClientKey(description: string): Promise<{ api_key: string }> {
    return send<{ api_key: string }>("/api/clients/keys", "POST", { description });
  },

  async deleteClientKey(id: number): Promise<void> {
    await send<{ success: boolean }>(`/api/clients/keys/${id}`, "DELETE");
  },
};
