'use client'

import { useState } from 'react'
import { api } from '@/lib/api'
import { SettingsSection } from './settings-section'

export function SecuritySection({ onSaved }: { onSaved: (msg: string) => void }) {
  const [current, setCurrent] = useState('')
  const [next, setNext] = useState('')
  const [confirm, setConfirm] = useState('')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const mismatch = confirm.length > 0 && next !== confirm
  const canSubmit =
    current.length > 0 &&
    next.length > 0 &&
    confirm.length > 0 &&
    !mismatch &&
    !saving

  async function submit() {
    if (!canSubmit) return
    setSaving(true)
    setError(null)
    try {
      await api.changePassword(current, next)
      setCurrent('')
      setNext('')
      setConfirm('')
      onSaved('密码已更新')
    } catch (err) {
      setError(err instanceof Error ? err.message : '更新失败')
    } finally {
      setSaving(false)
    }
  }

  return (
    <SettingsSection
      icon="password"
      title="账户安全"
      description="修改管理员登录密码"
    >
      <md-outlined-text-field
        label="当前密码"
        type="password"
        value={current}
        oninput={(e: Event) => setCurrent((e.target as HTMLInputElement).value)}
      >
        <md-icon slot="leading-icon">lock</md-icon>
      </md-outlined-text-field>
      <md-outlined-text-field
        label="新密码"
        type="password"
        value={next}
        oninput={(e: Event) => setNext((e.target as HTMLInputElement).value)}
      >
        <md-icon slot="leading-icon">lock_reset</md-icon>
      </md-outlined-text-field>
      <md-outlined-text-field
        label="确认新密码"
        type="password"
        value={confirm}
        error={mismatch || undefined}
        error-text="两次输入的密码不一致"
        oninput={(e: Event) => setConfirm((e.target as HTMLInputElement).value)}
      >
        <md-icon slot="leading-icon">check_circle</md-icon>
      </md-outlined-text-field>

      {error ? (
        <p className="md-typescale-body-small" style={{ color: 'var(--md-sys-color-error, #b3261e)' }}>
          {error}
        </p>
      ) : null}

      <div className="settings-actions">
        <md-filled-button disabled={!canSubmit || undefined} onclick={submit}>
          <md-icon slot="icon">save</md-icon>
          {saving ? '更新中…' : '更新密码'}
        </md-filled-button>
      </div>
    </SettingsSection>
  )
}
