'use client'

import { useEffect, useState } from 'react'
import { api } from '@/lib/api'
import { SettingsSection } from './settings-section'

interface DangerConfig {
  key: 'clear' | 'reset'
  label: string
  icon: string
  headline: string
  body: string
  confirmLabel: string
  toast: string
  run: () => Promise<void>
}

const DANGERS: Omit<DangerConfig, 'run'>[] = [
  {
    key: 'clear',
    label: '清空活动日志',
    icon: 'delete_sweep',
    headline: '清空活动日志？',
    body: '将永久删除所有已记录的活动事件，此操作不可撤销。',
    confirmLabel: '确认清空',
    toast: '活动日志已清空',
  },
  {
    key: 'reset',
    label: '重置运行时统计',
    icon: 'restart_alt',
    headline: '重置运行时统计？',
    body: '累计上报消息数等运行时统计将归零，此操作不可撤销。',
    confirmLabel: '确认重置',
    toast: '运行时统计已重置',
  },
]

export function DataSection(props: { onDone: (msg: string) => void }) {
  const [totalEvents, setTotalEvents] = useState<number | null>(null)
  const [totalMessages, setTotalMessages] = useState<number | null>(null)
  const [pending, setPending] = useState<(Omit<DangerConfig, 'run'> & { run: () => Promise<void> }) | null>(null)
  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function refresh() {
    try {
      const data = await api.getData()
      setTotalEvents(data.total_events)
      setTotalMessages(data.total_messages)
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载数据失败')
    }
  }

  useEffect(() => {
    refresh()
  }, [])

  async function confirm() {
    if (!pending || running) return
    setRunning(true)
    setError(null)
    try {
      await pending.run()
      setPending(null)
      props.onDone(pending.toast)
      await refresh()
    } catch (err) {
      setError(err instanceof Error ? err.message : '操作失败')
    } finally {
      setRunning(false)
    }
  }

  const actions: Record<DangerConfig['key'], () => Promise<void>> = {
    clear: async () => {
      await api.clearActivity()
    },
    reset: async () => {
      await api.resetStats()
    },
  }

  return (
    <SettingsSection
      icon="database"
      title="数据管理"
      description="查看与清理已采集的数据"
    >
      <div className="stat-readout">
        <span className="md-typescale-label-medium stat-readout-label">
          活动事件总数
        </span>
        <span className="md-typescale-display-small stat-readout-value">
          {totalEvents === null ? '—' : totalEvents.toLocaleString('zh-CN')}
        </span>
      </div>

      <div className="stat-readout">
        <span className="md-typescale-label-medium stat-readout-label">
          累计上报消息数
        </span>
        <span className="md-typescale-display-small stat-readout-value">
          {totalMessages === null ? '—' : totalMessages.toLocaleString('zh-CN')}
        </span>
      </div>

      <div className="danger-zone">
        {DANGERS.map((d) => (
          <div className="danger-row" key={d.key}>
            <span className="danger-row-text">
              <span className="md-typescale-body-large">{d.label}</span>
              <span className="md-typescale-body-small danger-row-hint">
                {d.body}
              </span>
            </span>
            <md-outlined-button
              class="danger-button"
              onClick={() => setPending({ ...d, run: actions[d.key] })}
            >
              <md-icon slot="icon">{d.icon}</md-icon>
              {d.label}
            </md-outlined-button>
          </div>
        ))}
      </div>

      {error ? (
        <p className="md-typescale-body-small" style={{ color: 'var(--md-sys-color-error, #b3261e)' }}>
          {error}
        </p>
      ) : null}

      <md-dialog
        open={pending ? true : undefined}
        onclose={() => (running ? undefined : setPending(null))}
      >
        <md-icon slot="icon">warning</md-icon>
        <div slot="headline">{pending?.headline}</div>
        <div slot="content" className="md-typescale-body-medium">
          {pending?.body}
        </div>
        <div slot="actions">
          <md-text-button
            disabled={running || undefined}
            onClick={() => setPending(null)}
          >
            取消
          </md-text-button>
          <md-filled-button
            class="danger-confirm"
            disabled={running || undefined}
            onClick={confirm}
          >
            {running ? '处理中…' : (pending?.confirmLabel ?? '确认')}
          </md-filled-button>
        </div>
      </md-dialog>
    </SettingsSection>
  )
}
