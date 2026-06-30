'use client'

import { useState } from 'react'
import { api, ApiError } from '@/lib/api'

export function LoginForm(props: { onSuccess: () => void }) {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [errorText, setErrorText] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  async function submit() {
    if (submitting) return
    if (username.trim().length === 0 || password.length === 0) {
      setErrorText('请输入用户名和密码')
      return
    }
    setSubmitting(true)
    setErrorText(null)
    try {
      await api.login(username.trim(), password)
      props.onSuccess()
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        setErrorText('用户名或密码不正确')
      } else {
        setErrorText(err instanceof Error ? err.message : '登录失败')
      }
    } finally {
      setSubmitting(false)
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.isComposing && e.keyCode !== 229) submit()
  }

  return (
    <div className="login-shell">
      <md-elevated-card className="login-card">
        <div className="login-head">
          <span className="login-badge" aria-hidden="true">
            <md-icon>shield_person</md-icon>
          </span>
          <h1 className="md-typescale-headline-small">控制面板登录</h1>
          <p className="md-typescale-body-medium login-sub">
            请输入管理员凭据以管理上报与转发配置
          </p>
        </div>

        <div className="login-fields">
          <md-outlined-text-field
            label="用户名"
            value={username}
            oninput={(e: Event) =>
              setUsername((e.target as HTMLInputElement).value)
            }
            onkeydown={onKeyDown}
          >
            <md-icon slot="leading-icon">person</md-icon>
          </md-outlined-text-field>

          <md-outlined-text-field
            label="密码"
            type="password"
            value={password}
            error={errorText ? true : undefined}
            error-text={errorText ?? undefined}
            oninput={(e: Event) => {
              setPassword((e.target as HTMLInputElement).value)
              setErrorText(null)
            }}
            onkeydown={onKeyDown}
          >
            <md-icon slot="leading-icon">lock</md-icon>
          </md-outlined-text-field>
        </div>

        <md-filled-button
          className="login-submit"
          disabled={submitting || undefined}
          onClick={submit}
        >
          <md-icon slot="icon">login</md-icon>
          {submitting ? '登录中…' : '登录'}
        </md-filled-button>

        <p className="md-typescale-body-small login-hint">
          初始密码在服务端首次启动时打印于控制台
        </p>
      </md-elevated-card>
    </div>
  )
}
