export function bytesToDataUri(data: number[] | undefined): string | undefined {
  if (!data || data.length === 0)
    return undefined

  let mime = 'application/octet-stream'
  if (data[0] === 0x89 && data[1] === 0x50 && data[2] === 0x4E && data[3] === 0x47)
    mime = 'image/png'
  else if (data[0] === 0xFF && data[1] === 0xD8 && data[2] === 0xFF)
    mime = 'image/jpeg'

  const chunkSize = 8192
  const parts: string[] = []
  for (let index = 0; index < data.length; index += chunkSize) {
    parts.push(String.fromCharCode(...data.slice(index, index + chunkSize)))
  }

  return `data:${mime};base64,${btoa(parts.join(''))}`
}

export function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0)
    return '--:--'

  const roundedSeconds = Math.round(seconds)
  return `${String(Math.floor(roundedSeconds / 60)).padStart(2, '0')}:${String(roundedSeconds % 60).padStart(2, '0')}`
}

export function nowLabel(): string {
  const date = new Date()
  return `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}:${String(date.getSeconds()).padStart(2, '0')}.${String(date.getMilliseconds()).padStart(3, '0')}`
}

export function normalizeReporterConfig(config: Partial<import('../types').ReporterConfig>): import('../types').ReporterConfig {
  return {
    protocol: config.protocol ?? 'native',
    enable_media_reporting: config.enable_media_reporting ?? false,
    native: {
      ws_url: config.native?.ws_url ?? '',
      token: config.native?.token ?? '',
    },
    mix_space: {
      endpoint: config.mix_space?.endpoint ?? '',
      method: config.mix_space?.method ?? 'POST',
      token: config.mix_space?.token ?? '',
    },
    s3: {
      enabled: config.s3?.enabled ?? false,
      bucket: config.s3?.bucket ?? '',
      region: config.s3?.region ?? 'us-east-1',
      access_key: config.s3?.access_key ?? '',
      secret_key: config.s3?.secret_key ?? '',
      endpoint: config.s3?.endpoint ?? '',
      custom_domain: config.s3?.custom_domain ?? '',
      key_template: config.s3?.key_template ?? '{kind}/{Y}/{M}/{D}/{SHA}.{ext}',
      lifecycle_days: config.s3?.lifecycle_days ?? 0,
    },
  }
}
