'use client'

import { useEffect, useState } from 'react'
import { api, type AccessSettings } from '@/lib/api'
import { FieldRow, SettingsSection } from './settings-section'

export function AccessSection(props: { onSaved: (msg: string) => void }) {
  const [settings, setSettings] = useState<AccessSettings>({
    accept_desktop: true,
    accept_mobile: true,
    activity_log_limit: 120,
  })
  const [loaded, setLoaded] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    api
      .getAccess()
      .then((data) => {
        if (!cancelled) {
          setSettings(data)
          setLoaded(true)
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : '加载接入控制失败')
          setLoaded(true)
        }
      })
    return () => {
      cancelled = true
    }
  }, [])

  function update(patch: Partial<AccessSettings>) {
    setSettings((prev) => ({ ...prev, ...patch }))
  }

  async function save() {
    if (saving) return
    setSaving(true)
    setError(null)
    try {
      const saved = await api.saveAccess(settings)
      setSettings(saved)
      props.onSaved('接入控制配置已保存')
    } catch (err) {
      setError(err instanceof Error ? err.message : '保存失败')
    } finally {
      setSaving(false)
    }
  }

  return (
    <SettingsSection
      icon="lan"
      title="接入控制"
      description="管理客户端连接与日志保留策略"
    >
      {!loaded ? (
        <p className="md-typescale-body-medium">加载中…</p>
      ) : (
        <>
          <FieldRow
            label="接受 Desktop 连接"
            hint="允许 Desktop 客户端上报状态"
            control={
              <md-switch
                selected={settings.accept_desktop || undefined}
                onchange={(e: Event) =>
                  update({
                    accept_desktop: (e.target as unknown as { selected: boolean }).selected,
                  })
                }
                aria-label="接受 Desktop 连接"
              />
            }
          />
          <FieldRow
            label="接受 Mobile 连接"
            hint="允许 Android / 移动端客户端上报状态"
            control={
              <md-switch
                selected={settings.accept_mobile || undefined}
                onchange={(e: Event) =>
                  update({
                    accept_mobile: (e.target as unknown as { selected: boolean }).selected,
                  })
                }
                aria-label="接受 Mobile 连接"
              />
            }
          />
          <md-outlined-text-field
            label="活动日志保留条数"
            type="number"
            min={1}
            value={String(settings.activity_log_limit)}
            supporting-text="默认 120 条，超出后自动滚动覆盖"
            oninput={(e: Event) => {
              const raw = (e.target as HTMLInputElement).value
              const n = Math.max(1, Math.floor(Number(raw) || 0))
              update({ activity_log_limit: n })
            }}
          >
            <md-icon slot="leading-icon">history</md-icon>
          </md-outlined-text-field>

          {error ? (
            <p className="md-typescale-body-small" style={{ color: 'var(--md-sys-color-error, #b3261e)' }}>
              {error}
            </p>
          ) : null}

          <div className="settings-actions">
            <md-filled-button disabled={saving || undefined} onClick={save}>
              <md-icon slot="icon">save</md-icon>
              {saving ? '保存中…' : '保存'}
            </md-filled-button>
          </div>
        </>
      )}
    </SettingsSection>
  )
}
