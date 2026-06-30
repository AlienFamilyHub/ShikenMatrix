'use client'

import { apiUrl } from '@/lib/api'
import { type ActivityInfo, present, anyPresent } from '@/lib/status-data'

export function ActivityCard({ activity }: { activity?: ActivityInfo | null }) {
  if (
    !activity ||
    !anyPresent(
      activity.process_name,
      activity.title,
      activity.icon_url,
      activity.app_id,
    )
  ) {
    return null
  }

  return (
    <md-outlined-card class="info-card activity-card">
      <header className="card-header">
        <md-icon>apps</md-icon>
        <h3 className="md-typescale-title-medium">当前活动</h3>
      </header>

      <div className="activity-body">
        {present(activity.icon_url) && (
          <img
            className="activity-icon"
            src={apiUrl(activity.icon_url as string)}
            alt=""
            width={48}
            height={48}
          />
        )}
        <div className="activity-text">
          {present(activity.process_name) && (
            <span className="md-typescale-title-medium activity-name">
              {activity.process_name}
            </span>
          )}
          {present(activity.title) && (
            <span className="md-typescale-body-medium activity-window">
              {activity.title}
            </span>
          )}
          {present(activity.app_id) && (
            <code className="md-typescale-label-small activity-appid">
              {activity.app_id}
            </code>
          )}
        </div>
      </div>
    </md-outlined-card>
  )
}
