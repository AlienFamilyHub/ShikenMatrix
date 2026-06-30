'use client'

import {
  type OnlineStatus,
  present,
  anyPresent,
  clientKindLabel,
  formatRelativeTime,
  formatAbsoluteTime,
} from '@/lib/status-data'

export function StatusCard({ status }: { status?: OnlineStatus | null }) {
  if (
    !status ||
    !anyPresent(
      status.is_online,
      status.client_kind,
      status.device_info,
      status.last_activity_at,
    )
  ) {
    return null
  }

  const online = status.is_online === true
  const kind = clientKindLabel(status.client_kind)
  const lastActive = formatRelativeTime(status.last_activity_at)
  const lastSeen = formatAbsoluteTime(status.last_activity_at)

  return (
    <md-elevated-card
      class={`status-card ${online ? 'is-online' : 'is-offline'}`}
    >
      <div className="status-card-main">
        <span className="status-dot" aria-hidden="true" />
        <div className="status-headline">
          <span className="md-typescale-label-large status-eyebrow">
            连接状态
          </span>
          <h2 className="md-typescale-headline-medium status-title">
            {online ? '在线' : '设备离线'}
          </h2>
        </div>

        {online && present(kind) && (
          <span className="status-kind md-typescale-label-large">
            <md-icon>
              {status.client_kind === 'mobile'
                ? 'smartphone'
                : 'desktop_windows'}
            </md-icon>
            {kind}
          </span>
        )}
      </div>

      <div className="status-meta">
        {online ? (
          <>
            {present(status.device_info) && (
              <div className="status-meta-item">
                <span className="md-typescale-label-medium meta-label">
                  设备
                </span>
                <span className="md-typescale-body-large meta-value">
                  {status.device_info}
                </span>
              </div>
            )}
            {present(lastActive) && (
              <div className="status-meta-item">
                <span className="md-typescale-label-medium meta-label">
                  最后活跃
                </span>
                <span
                  className="md-typescale-body-large meta-value"
                  suppressHydrationWarning
                >
                  {lastActive}
                </span>
              </div>
            )}
          </>
        ) : (
          present(lastSeen) && (
            <div className="status-meta-item">
              <span className="md-typescale-label-medium meta-label">
                最后在线
              </span>
              <span className="md-typescale-body-large meta-value">
                {lastSeen}
              </span>
            </div>
          )
        )}
      </div>
    </md-elevated-card>
  )
}
