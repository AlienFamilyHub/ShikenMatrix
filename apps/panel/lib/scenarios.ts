import type { ClientKind, Snapshot } from './status-data'

export interface Scenario {
  id: string
  label: string
  icon: string
  data: Snapshot
}

/** Offline placeholder used before the first device event arrives. */
function offlineScenario(kind: ClientKind): Scenario {
  return {
    id: kind,
    label: kind === 'desktop' ? 'Desktop · 离线' : 'Android · 离线',
    icon: kind === 'desktop' ? 'desktop_windows' : 'smartphone',
    data: {
      status: {
        is_online: false,
        client_kind: kind,
        device_info: null,
        last_activity_at: null,
      },
      activity: null,
      media: null,
      device: null,
      stats: { total_messages: 0 },
    },
  }
}

export function scenarioFromSnapshot(kind: ClientKind, snapshot: Snapshot): Scenario {
  const isDesktop = kind === 'desktop'
  const name = isDesktop ? 'Desktop' : 'Android'

  return {
    id: kind,
    label: snapshot.status?.is_online ? name : `${name} · 离线`,
    icon: isDesktop ? 'desktop_windows' : 'smartphone',
    data: snapshot,
  }
}

/** Initial scenarios shown while the first device events are in flight. */
export const initialScenarios: Scenario[] = [
  offlineScenario('desktop'),
  offlineScenario('mobile'),
]
