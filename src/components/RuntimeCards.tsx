import type { MediaView, WindowView } from '../types'
import { Show } from 'solid-js'
import IconComputer from '~icons/mingcute/computer-line'
import IconInbox from '~icons/mingcute/inbox-line'
import IconMusic from '~icons/mingcute/music-2-line'
import { formatDuration } from '../lib/format'

interface RuntimeCardsProps {
  windowInfo: WindowView | null
  mediaInfo: MediaView | null
}

export function RuntimeCards(props: RuntimeCardsProps) {
  return (
    <section class="info-section">
      <h2 class="section-title">实时状态</h2>

      <div class="info-card">
        <Show
          when={props.windowInfo}
          fallback={(
            <div class="empty-text">
              <IconInbox class="empty-icon" />
              <span>暂无前台窗口数据</span>
            </div>
          )}
        >
          {windowInfo => (
            <>
              <Show when={windowInfo().iconSrc} fallback={<div class="info-icon" />}>
                {source => <img class="info-icon" src={source()} alt="icon" />}
              </Show>
              <div class="info-details">
                <div class="info-type">
                  <IconComputer class="info-type-icon" />
                  当前窗口
                </div>
                <div class="info-title">{windowInfo().title || '未知窗口'}</div>
                <div class="info-sub">
                  {windowInfo().process_name}
                  {' '}
                  · PID
                  {' '}
                  {windowInfo().pid}
                </div>
              </div>
            </>
          )}
        </Show>
      </div>

      <div class="info-card">
        <Show
          when={props.mediaInfo}
          fallback={(
            <div class="empty-text">
              <IconInbox class="empty-icon" />
              <span>暂无媒体播放数据</span>
            </div>
          )}
        >
          {mediaInfo => (
            <>
              <Show when={mediaInfo().artworkSrc} fallback={<div class="info-icon" />}>
                {source => <img class="info-icon" src={source()} alt="artwork" />}
              </Show>
              <div class="info-details">
                <div class="info-type">
                  <IconMusic class="info-type-icon" />
                  媒体播放
                </div>
                <div class="info-title">{mediaInfo().title || '未知媒体'}</div>
                <div class="info-sub">
                  {mediaInfo().artist}
                  {' '}
                  ·
                  {mediaInfo().playing ? '播放中' : '已暂停'}
                  {' '}
                  /
                  {formatDuration(mediaInfo().elapsed_time)}
                </div>
              </div>
            </>
          )}
        </Show>
      </div>
    </section>
  )
}
