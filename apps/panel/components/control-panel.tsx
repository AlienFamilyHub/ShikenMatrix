'use client'

import { useEffect, useState } from 'react'
import { api } from '@/lib/api'
import { LoginForm } from './control/login-form'
import { UpstreamSection } from './control/upstream-section'
import { AccessSection } from './control/access-section'
import { SecuritySection } from './control/security-section'
import { DataSection } from './control/data-section'
import { ClientsSection } from './control/clients-section'

type AuthState = 'checking' | 'anonymous' | 'authed'

export function ControlPanel() {
  const [authState, setAuthState] = useState<AuthState>('checking')
  const [toast, setToast] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    api
      .checkAuth()
      .then((ok) => {
        if (!cancelled) setAuthState(ok ? 'authed' : 'anonymous')
      })
      .catch(() => {
        if (!cancelled) setAuthState('anonymous')
      })
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    if (!toast) return
    const t = setTimeout(() => setToast(null), 2600)
    return () => clearTimeout(t)
  }, [toast])

  if (authState === 'checking') {
    return (
      <div className="control-panel">
        <p className="md-typescale-body-medium">正在验证登录状态…</p>
      </div>
    )
  }

  if (authState === 'anonymous') {
    return (
      <LoginForm
        onSuccess={() => setAuthState('authed')}
      />
    )
  }

  return (
    <div className="control-panel">
      <div className="control-head">
        <div className="control-head-titles">
          <h1 className="md-typescale-headline-medium">控制面板</h1>
          <p className="md-typescale-body-medium control-head-sub">
            管理上报转发、接入控制与账户安全
          </p>
        </div>
        <md-outlined-button
          onclick={async () => {
            await api.logout()
            setAuthState('anonymous')
          }}
        >
          <md-icon slot="icon">logout</md-icon>
          退出登录
        </md-outlined-button>
      </div>

      <div className="control-grid">
        <UpstreamSection onSaved={setToast} />
        <AccessSection onSaved={setToast} />
        <ClientsSection onSaved={setToast} />
        <SecuritySection onSaved={setToast} />
        <DataSection onDone={setToast} />
      </div>

      {toast ? (
        <div className="snackbar md-typescale-body-medium" role="status">
          <md-icon class="snackbar-icon">check_circle</md-icon>
          {toast}
        </div>
      ) : null}
    </div>
  )
}
