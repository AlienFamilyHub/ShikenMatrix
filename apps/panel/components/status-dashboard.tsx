'use client'

import { useEffect, useRef, useState } from 'react'
import {
  initialScenarios,
  scenarioFromSnapshot,
  type Scenario,
} from '@/lib/scenarios'
import { apiUrl } from '@/lib/api'
import { type ClientKind, type Snapshot } from '@/lib/status-data'
import { StatusCard } from './status-card'
import { ActivityCard } from './activity-card'
import { MediaCard } from './media-card'
import { DeviceCard } from './device-card'

const deviceEventSources: Array<{ kind: ClientKind, url: string }> = [
  { kind: 'desktop', url: '/api/share/desktop/events' },
  { kind: 'mobile', url: '/api/share/mobile/events' },
]

export function StatusDashboard() {
  const [scenarios, setScenarios] = useState<Scenario[]>(initialScenarios)
  const [activeId, setActiveId] = useState(initialScenarios[0].id)
  const activeIdRef = useRef(activeId)
  const userSelectedRef = useRef(false)

  useEffect(() => {
    activeIdRef.current = activeId
  }, [activeId])

  useEffect(() => {
    function applySnapshot(kind: ClientKind, snapshot: Snapshot) {
      const nextScenario = scenarioFromSnapshot(kind, snapshot)

      setScenarios((prev) => {
        const next = prev.map((scenario) =>
          scenario.id === kind ? nextScenario : scenario,
        )
        const firstOnlineScenario = next.find(
          (scenario) => scenario.data.status?.is_online,
        )

        // 用户未主动选择前，优先展示真正在线的设备，避免默认停在离线 Desktop。
        if (!userSelectedRef.current && firstOnlineScenario) {
          setActiveId(firstOnlineScenario.id)
        }
        // 即使选中项变化，也要保持 tab 文案和数据为最新。
        return next
      })
    }

    const eventSources = deviceEventSources.map(({ kind, url }) => {
      const eventSource = new EventSource(apiUrl(url))
      eventSource.addEventListener(kind, (event) => {
        try {
          applySnapshot(kind, JSON.parse(event.data) as Snapshot)
        } catch {
          /* keep last known state when a malformed event slips through */
        }
      })
      return eventSource
    })

    return () => {
      for (const eventSource of eventSources) {
        eventSource.close()
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const scenario = scenarios.find((s) => s.id === activeId) ?? scenarios[0]
  const { data } = scenario

  return (
    <div className="dashboard">
      <div className="scenario-switcher" role="tablist" aria-label="示例数据状态">
        {scenarios.map((s) => {
          const active = s.id === activeId
          return (
            <span
              key={s.id}
              className="scenario-btn-wrap"
              onClick={() => {
                userSelectedRef.current = true
                setActiveId(s.id)
              }}
            >
              <md-outlined-button
                role="tab"
                aria-selected={active}
                class={active ? 'scenario-btn active' : 'scenario-btn'}
              >
                <md-icon slot="icon">{active ? 'check' : s.icon}</md-icon>
                {s.label}
              </md-outlined-button>
            </span>
          )
        })}
      </div>

      <StatusCard status={data.status} />

      <div className="card-grid">
        <ActivityCard activity={data.activity} />
        <MediaCard media={data.media} />
        <DeviceCard device={data.device} />
      </div>
    </div>
  )
}
