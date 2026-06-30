'use client'

import { useEffect, useState } from 'react'
import { api, type ClientKeyEntry } from '@/lib/api'
import { SettingsSection } from './settings-section'

export function ClientsSection(props: { onSaved: (msg: string) => void }) {
  const [keys, setKeys] = useState<ClientKeyEntry[]>([])
  const [loaded, setLoaded] = useState(false)
  const [description, setDescription] = useState('')
  const [newKey, setNewKey] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function refresh() {
    try {
      const list = await api.getClientKeys()
      setKeys(list)
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载客户端密钥失败')
    } finally {
      setLoaded(true)
    }
  }

  useEffect(() => {
    refresh()
  }, [])

  async function create() {
    const desc = description.trim()
    if (!desc || busy) return
    setBusy(true)
    setError(null)
    try {
      const { api_key } = await api.createClientKey(desc)
      setNewKey(api_key)
      setDescription('')
      await refresh()
      props.onSaved('客户端密钥已创建')
    } catch (err) {
      setError(err instanceof Error ? err.message : '创建失败')
    } finally {
      setBusy(false)
    }
  }

  async function remove(id: number) {
    if (busy) return
    setBusy(true)
    setError(null)
    try {
      await api.deleteClientKey(id)
      await refresh()
      props.onSaved('客户端密钥已吊销')
    } catch (err) {
      setError(err instanceof Error ? err.message : '吊销失败')
    } finally {
      setBusy(false)
    }
  }

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text)
      props.onSaved('已复制到剪贴板')
    } catch {
      setError('复制失败，请手动选择文本')
    }
  }

  return (
    <SettingsSection
      icon="vpn_key"
      title="客户端密钥"
      description="为 Desktop / Android 客户端签发上报凭据"
    >
      {!loaded ? (
        <p className="md-typescale-body-medium">加载中…</p>
      ) : (
        <>
          <div className="client-create-row">
            <md-outlined-text-field
              label="描述（如：我的笔记本）"
              value={description}
              oninput={(e: Event) => setDescription((e.target as HTMLInputElement).value)}
              onkeydown={(e: KeyboardEvent) => {
                if (e.key === 'Enter' && !e.isComposing && e.keyCode !== 229) create()
              }}
            >
              <md-icon slot="leading-icon">label</md-icon>
            </md-outlined-text-field>
            <md-filled-button disabled={busy || !description.trim() || undefined} onClick={create}>
              <md-icon slot="icon">add</md-icon>
              新建
            </md-filled-button>
          </div>

          {newKey ? (
            <div className="client-newkey">
              <span className="md-typescale-body-medium">
                密钥已生成，仅展示一次，请立即复制并填入客户端：
              </span>
              <code className="client-newkey-value">{newKey}</code>
              <div className="client-newkey-actions">
                <md-outlined-button onClick={() => copy(newKey)}>
                  <md-icon slot="icon">content_copy</md-icon>
                  复制
                </md-outlined-button>
                <md-text-button onClick={() => setNewKey(null)}>知道了</md-text-button>
              </div>
            </div>
          ) : null}

          {keys.length === 0 ? (
            <p className="md-typescale-body-small">尚无客户端密钥。</p>
          ) : (
            <ul className="client-list">
              {keys.map((k) => (
                <li className="client-row" key={k.id}>
                  <span className="client-row-text">
                    <span className="md-typescale-body-large">{k.description}</span>
                    <code className="md-typescale-label-small client-row-key">
                      {k.api_key.slice(0, 10)}…
                    </code>
                  </span>
                  <md-icon-button onClick={() => remove(k.id)} aria-label="吊销">
                    <md-icon>delete</md-icon>
                  </md-icon-button>
                </li>
              ))}
            </ul>
          )}

          {error ? (
            <p className="md-typescale-body-small" style={{ color: 'var(--md-sys-color-error, #b3261e)' }}>
              {error}
            </p>
          ) : null}
        </>
      )}
    </SettingsSection>
  )
}
