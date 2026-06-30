'use client'

import { type Stats, present } from '@/lib/status-data'

export function StatsCard({ stats }: { stats?: Stats | null }) {
  if (!stats || !present(stats.total_messages)) {
    return null
  }

  return (
    <md-filled-card class="info-card stats-card">
      <header className="card-header">
        <md-icon>insights</md-icon>
        <h3 className="md-typescale-title-medium">统计</h3>
      </header>
      <div className="stats-body">
        <span className="md-typescale-display-small stats-value">
          {(stats.total_messages as number).toLocaleString('zh-CN')}
        </span>
        <span className="md-typescale-label-medium stats-label">
          累计上报消息数
        </span>
      </div>
    </md-filled-card>
  )
}
