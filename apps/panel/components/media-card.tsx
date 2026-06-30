'use client'

import { useEffect, useState } from 'react'
import { apiUrl } from '@/lib/api'
import {
  type MediaInfo,
  present,
  anyPresent,
  formatDuration,
} from '@/lib/status-data'

export function MediaCard({ media }: { media?: MediaInfo | null }) {
  const [localElapsedTime, setLocalElapsedTime] = useState(media?.elapsed_time ?? null)

  useEffect(() => {
    setLocalElapsedTime(media?.elapsed_time ?? null)
  }, [media?.elapsed_time, media?.title, media?.artist])

  useEffect(() => {
    if (!media?.playing || typeof media.elapsed_time !== 'number') return

    const startedAt = Date.now()
    const baseElapsedTime = media.elapsed_time
    const timer = window.setInterval(() => {
      const nextElapsedTime = baseElapsedTime + (Date.now() - startedAt) / 1000
      const duration = media.duration
      setLocalElapsedTime(
        typeof duration === 'number' && duration > 0
          ? Math.min(nextElapsedTime, duration)
          : nextElapsedTime,
      )
    }, 1000)

    return () => window.clearInterval(timer)
  }, [media?.playing, media?.duration, media?.elapsed_time, media?.title, media?.artist])

  if (
    !media ||
    !anyPresent(
      media.title,
      media.artist,
      media.album,
      media.duration,
      localElapsedTime,
      media.playing,
      media.artwork_url,
    )
  ) {
    return null
  }

  const elapsed = formatDuration(localElapsedTime)
  const total = formatDuration(media.duration)
  const hasProgress =
    present(localElapsedTime) &&
    present(media.duration) &&
    (media.duration as number) > 0
  const progress = hasProgress
    ? Math.min(1, (localElapsedTime as number) / (media.duration as number))
    : null
  const showPlaying = present(media.playing)

  return (
    <md-outlined-card class="info-card media-card">
      <header className="card-header">
        <md-icon>music_note</md-icon>
        <h3 className="md-typescale-title-medium">媒体播放</h3>
        {showPlaying && (
          <span
            className={`media-state md-typescale-label-medium ${
              media.playing ? 'is-playing' : 'is-paused'
            }`}
          >
            <md-icon>{media.playing ? 'play_arrow' : 'pause'}</md-icon>
            {media.playing ? '播放中' : '已暂停'}
          </span>
        )}
      </header>

      <div className="media-body">
        {present(media.artwork_url) && (
          <img
            className="media-artwork"
            src={apiUrl(media.artwork_url as string)}
            alt=""
            width={88}
            height={88}
          />
        )}
        <div className="media-text">
          {present(media.title) && (
            <span className="md-typescale-title-medium media-title">
              {media.title}
            </span>
          )}
          {present(media.artist) && (
            <span className="md-typescale-body-medium media-artist">
              {media.artist}
            </span>
          )}
          {present(media.album) && (
            <span className="md-typescale-body-small media-album">
              {media.album}
            </span>
          )}
        </div>
      </div>

      {(progress !== null || present(elapsed) || present(total)) && (
        <div className="media-progress">
          {progress !== null && <md-linear-progress value={progress} />}
          {(present(elapsed) || present(total)) && (
            <div className="media-time md-typescale-label-small">
              <span>{elapsed ?? '0:00'}</span>
              {present(total) && <span>{total}</span>}
            </div>
          )}
        </div>
      )}
    </md-outlined-card>
  )
}
