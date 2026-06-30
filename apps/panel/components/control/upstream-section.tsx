'use client'

import { useEffect, useState } from 'react'
import { api, type UpstreamSettings } from '@/lib/api'
import { SettingsSection, FieldRow } from './settings-section'

type Protocol = 'native' | 'mix_space'

const EMPTY_UPSTREAM: UpstreamSettings = {
  protocol: 'native',
  enable_media_reporting: true,
  native_ws_url: '',
  native_token: '',
  mix_space_endpoint: '',
  mix_space_method: 'POST',
  mix_space_token: '',
  s3_enabled: false,
  s3_bucket: '',
  s3_region: 'us-east-1',
  s3_access_key: '',
  s3_secret_key: '',
  s3_endpoint: '',
  s3_custom_domain: '',
  s3_key_template: 'uploads/{year}/{month}/{filename}',
}

export function UpstreamSection({ onSaved }: { onSaved: (msg: string) => void }) {
  const [settings, setSettings] = useState<UpstreamSettings>(EMPTY_UPSTREAM)
  const [loaded, setLoaded] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    api
      .getUpstream()
      .then((data) => {
        if (!cancelled) {
          setSettings(data)
          setLoaded(true)
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : '加载上游配置失败')
          setLoaded(true)
        }
      })
    return () => {
      cancelled = true
    }
  }, [])

  function update(patch: Partial<UpstreamSettings>) {
    setSettings((prev) => ({ ...prev, ...patch }))
  }

  async function save() {
    if (saving) return
    setSaving(true)
    setError(null)
    try {
      await api.saveUpstream(settings)
      onSaved('上游转发配置已保存')
    } catch (err) {
      setError(err instanceof Error ? err.message : '保存失败')
    } finally {
      setSaving(false)
    }
  }

  const protocol = settings.protocol as Protocol

  return (
    <SettingsSection
      icon="cloud_upload"
      title="上游转发"
      description="配置状态数据转发到的上游服务"
    >
      {!loaded ? (
        <p className="md-typescale-body-medium">加载中…</p>
      ) : (
        <>
          <md-outlined-select
            label="协议"
            value={protocol}
            onchange={(e: Event) =>
              update({ protocol: (e.target as HTMLSelectElement).value as Protocol })
            }
          >
            <md-select-option value="native" selected={protocol === 'native' || undefined}>
              <div slot="headline">Native</div>
            </md-select-option>
            <md-select-option value="mix_space" selected={protocol === 'mix_space' || undefined}>
              <div slot="headline">Mix-Space</div>
            </md-select-option>
          </md-outlined-select>

          <FieldRow
            label="启用媒体上报"
            hint="上报正在播放的媒体信息"
            control={
              <md-switch
                selected={settings.enable_media_reporting || undefined}
                onchange={(e: Event) =>
                  update({
                    enable_media_reporting: (e.target as unknown as { selected: boolean }).selected,
                  })
                }
                aria-label="启用媒体上报"
              />
            }
          />

          {protocol === 'native' ? (
            <div className="field-group">
              <span className="md-typescale-label-large field-group-label">Native</span>
              <md-outlined-text-field
                label="WebSocket URL"
                value={settings.native_ws_url}
                placeholder="wss://example.com/ws"
                oninput={(e: Event) =>
                  update({ native_ws_url: (e.target as HTMLInputElement).value })
                }
              />
              <md-outlined-text-field
                label="Token"
                type="password"
                value={settings.native_token}
                oninput={(e: Event) =>
                  update({ native_token: (e.target as HTMLInputElement).value })
                }
              />
            </div>
          ) : (
            <div className="field-group">
              <span className="md-typescale-label-large field-group-label">Mix-Space</span>
              <md-outlined-text-field
                label="Endpoint"
                value={settings.mix_space_endpoint}
                placeholder="https://api.example.com"
                oninput={(e: Event) =>
                  update({ mix_space_endpoint: (e.target as HTMLInputElement).value })
                }
              />
              <md-outlined-select
                label="Method"
                value={settings.mix_space_method}
                onchange={(e: Event) =>
                  update({ mix_space_method: (e.target as HTMLSelectElement).value })
                }
              >
                <md-select-option value="POST" selected={settings.mix_space_method === 'POST' || undefined}>
                  <div slot="headline">POST</div>
                </md-select-option>
                <md-select-option value="PUT" selected={settings.mix_space_method === 'PUT' || undefined}>
                  <div slot="headline">PUT</div>
                </md-select-option>
                <md-select-option value="PATCH" selected={settings.mix_space_method === 'PATCH' || undefined}>
                  <div slot="headline">PATCH</div>
                </md-select-option>
              </md-outlined-select>
              <md-outlined-text-field
                label="Token"
                type="password"
                value={settings.mix_space_token}
                oninput={(e: Event) =>
                  update({ mix_space_token: (e.target as HTMLInputElement).value })
                }
              />
            </div>
          )}

          <div className="field-group">
            <FieldRow
              label="S3 存储"
              hint="启用后将媒体封面等资源上传到 S3"
              control={
                <md-switch
                  selected={settings.s3_enabled || undefined}
                  onchange={(e: Event) =>
                    update({ s3_enabled: (e.target as unknown as { selected: boolean }).selected })
                  }
                  aria-label="启用 S3 存储"
                />
              }
            />
            {settings.s3_enabled ? (
              <div className="s3-grid">
                <md-outlined-text-field
                  label="Bucket"
                  value={settings.s3_bucket}
                  oninput={(e: Event) =>
                    update({ s3_bucket: (e.target as HTMLInputElement).value })
                  }
                />
                <md-outlined-text-field
                  label="Region"
                  value={settings.s3_region}
                  oninput={(e: Event) =>
                    update({ s3_region: (e.target as HTMLInputElement).value })
                  }
                />
                <md-outlined-text-field
                  label="Access Key"
                  value={settings.s3_access_key}
                  oninput={(e: Event) =>
                    update({ s3_access_key: (e.target as HTMLInputElement).value })
                  }
                />
                <md-outlined-text-field
                  label="Secret Key"
                  type="password"
                  value={settings.s3_secret_key}
                  oninput={(e: Event) =>
                    update({ s3_secret_key: (e.target as HTMLInputElement).value })
                  }
                />
                <md-outlined-text-field
                  label="Endpoint"
                  value={settings.s3_endpoint}
                  placeholder="https://s3.amazonaws.com"
                  oninput={(e: Event) =>
                    update({ s3_endpoint: (e.target as HTMLInputElement).value })
                  }
                />
                <md-outlined-text-field
                  label="Custom Domain"
                  value={settings.s3_custom_domain}
                  placeholder="https://cdn.example.com"
                  oninput={(e: Event) =>
                    update({ s3_custom_domain: (e.target as HTMLInputElement).value })
                  }
                />
                <md-outlined-text-field
                  className="s3-grid-full"
                  label="Key Template"
                  value={settings.s3_key_template}
                  supporting-text="支持 {year} {month} {filename} 等占位符"
                  oninput={(e: Event) =>
                    update({ s3_key_template: (e.target as HTMLInputElement).value })
                  }
                />
              </div>
            ) : null}
          </div>

          {error ? (
            <p className="md-typescale-body-small" style={{ color: 'var(--md-sys-color-error, #b3261e)' }}>
              {error}
            </p>
          ) : null}

          <div className="settings-actions">
            <md-filled-button disabled={saving || undefined} onclick={save}>
              <md-icon slot="icon">save</md-icon>
              {saving ? '保存中…' : '保存'}
            </md-filled-button>
          </div>
        </>
      )}
    </SettingsSection>
  )
}
