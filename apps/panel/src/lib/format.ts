/** A value counts as present when it is not null/undefined and not blank. */
export function present(value: unknown): boolean {
  if (value === null || value === undefined)
    return false;
  if (typeof value === "string")
    return value.trim() !== "";
  return true;
}

/** True when at least one of the provided values is present. */
export function anyPresent(...values: unknown[]): boolean {
  return values.some(present);
}

/** Format seconds into `m:ss` (or `h:mm:ss`). */
export function formatDuration(seconds: number | null | undefined): string | null {
  if (typeof seconds !== "number" || !Number.isFinite(seconds) || seconds < 0) {
    return null;
  }
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = total % 60;
  const pad = (value: number) => value.toString().padStart(2, "0");
  return hours > 0
    ? `${hours}:${pad(minutes)}:${pad(rest)}`
    : `${minutes}:${pad(rest)}`;
}

/** Friendly relative label, e.g. "3 分钟前". */
export function formatRelativeTime(value: string | null | undefined): string | null {
  if (!present(value))
    return null;
  const then = new Date(value as string).getTime();
  if (Number.isNaN(then))
    return null;
  const seconds = Math.round((Date.now() - then) / 1000);
  if (seconds < 60)
    return "刚刚";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60)
    return `${minutes} 分钟前`;
  const hours = Math.round(minutes / 60);
  if (hours < 24)
    return `${hours} 小时前`;
  const days = Math.round(hours / 24);
  if (days < 30)
    return `${days} 天前`;
  return new Date(then).toLocaleDateString("zh-CN");
}

const absoluteFormat: Intl.DateTimeFormatOptions = {
  month: "long",
  day: "numeric",
  hour: "2-digit",
  minute: "2-digit",
};

/** Absolute local label, e.g. "5月3日 14:20". */
export function formatAbsoluteTime(value: string | null | undefined): string | null {
  if (!present(value))
    return null;
  const time = new Date(value as string);
  if (Number.isNaN(time.getTime()))
    return null;
  return time.toLocaleString("zh-CN", absoluteFormat);
}

/** Absolute local label from a unix timestamp in seconds. */
export function formatUnixTime(seconds: number | null | undefined): string | null {
  if (typeof seconds !== "number" || !Number.isFinite(seconds) || seconds <= 0)
    return null;
  return new Date(seconds * 1000).toLocaleString("zh-CN", absoluteFormat);
}

/** Compact human-readable duration for uptime-like values. */
export function formatUptime(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const days = Math.floor(total / 86400);
  const hours = Math.floor((total % 86400) / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  if (days > 0)
    return `${days} 天 ${hours} 小时`;
  if (hours > 0)
    return `${hours} 小时 ${minutes} 分`;
  return `${minutes} 分`;
}

/** Count label with thousands separators; em dash for unknown values. */
export function formatCount(value: number | null | undefined): string {
  if (typeof value !== "number" || !Number.isFinite(value))
    return "—";
  return value.toLocaleString("zh-CN");
}
