import type { UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { createMemo, createSignal, For, onCleanup, onMount, Show } from 'solid-js'
import IconComputer from '~icons/mingcute/computer-line'

import IconDelete from '~icons/mingcute/delete-2-line'
import IconMonitorOn from '~icons/mingcute/eye-2-line'
import IconMonitorOff from '~icons/mingcute/eye-close-line'
// --- Icons ---
import IconInbox from '~icons/mingcute/inbox-line'
import IconMusic from '~icons/mingcute/music-2-line'
import IconPlay from '~icons/mingcute/play-circle-line'
import IconShieldOn from '~icons/mingcute/safe-shield-line'
import IconSave from '~icons/mingcute/save-2-line'
import IconSearch from '~icons/mingcute/search-2-line'
import IconShieldOff from '~icons/mingcute/shield-line'
import IconStop from '~icons/mingcute/stop-circle-line'
import IconUpload from '~icons/mingcute/upload-2-line'
import IconWifiOn from '~icons/mingcute/wifi-line'
import IconWifiOff from '~icons/mingcute/wifi-off-line'
import appIconUrl from './assets/icon.svg'
import './App.css'

// --- Types ---
interface ReporterConfig {
  ws_url: string
  token: string
  enable_media_reporting: boolean
}

type CloseBehavior = 'hide_to_tray' | 'quit'

interface ConnectionStatus {
  is_monitoring: boolean
  is_reporting: boolean
  is_connected: boolean
  last_error: string | null
}

interface PermissionStatus {
  accessibility: boolean
  media: boolean
}

interface LogEntry {
  time: string
  level: 'Info' | 'Warn' | 'Error'
  message: string
}

interface WindowView {
  title: string
  process_name: string
  pid: number
  iconSrc?: string
}

interface MediaView {
  title: string
  artist: string
  album: string
  duration: number
  elapsed_time: number
  playing: boolean
  artworkSrc?: string
}

// --- Helpers ---
function bytesToDataUri(data: number[] | undefined): string | undefined {
  if (!data || data.length === 0)
    return undefined
  let mime = 'application/octet-stream'
  if (data[0] === 0x89 && data[1] === 0x50 && data[2] === 0x4E && data[3] === 0x47)
    mime = 'image/png'
  else if (data[0] === 0xFF && data[1] === 0xD8 && data[2] === 0xFF)
    mime = 'image/jpeg'

  const CHUNK = 8192
  const parts: string[] = []
  for (let i = 0; i < data.length; i += CHUNK) {
    parts.push(String.fromCharCode(...data.slice(i, i + CHUNK)))
  }
  return `data:${mime};base64,${btoa(parts.join(''))}`
}

function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0)
    return '--:--'
  const sec = Math.round(seconds)
  return `${String(Math.floor(sec / 60)).padStart(2, '0')}:${String(sec % 60).padStart(2, '0')}`
}

function nowLabel(): string {
  const d = new Date()
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}.${String(d.getMilliseconds()).padStart(3, '0')}`
}

const MAX_LOGS = 200

export default function App() {
  const [config, setConfig] = createSignal<ReporterConfig>({ ws_url: '', token: '', enable_media_reporting: false })
  const [status, setStatus] = createSignal<ConnectionStatus>({ is_monitoring: false, is_reporting: false, is_connected: false, last_error: null })
  const [permissions, setPermissions] = createSignal<PermissionStatus>({ accessibility: false, media: false })

  const [logs, setLogs] = createSignal<LogEntry[]>([])
  const [searchText, setSearchText] = createSignal('')

  const [windowInfo, setWindowInfo] = createSignal<WindowView | null>(null)
  const [mediaInfo, setMediaInfo] = createSignal<MediaView | null>(null)
  const [showCloseChoice, setShowCloseChoice] = createSignal(false)
  const [rememberCloseChoice, setRememberCloseChoice] = createSignal(true)

  const filteredLogs = createMemo(() => {
    const search = searchText().toLowerCase().trim()
    if (!search)
      return logs()
    return logs().filter(l => l.message.toLowerCase().includes(search))
  })

  const addLog = (level: string, message: string) => {
    setLogs((prev) => {
      const entry: LogEntry = { time: nowLabel(), level: level as LogEntry['level'], message }
      const next = [...prev, entry]
      return next.length > MAX_LOGS ? next.slice(next.length - MAX_LOGS) : next
    })
  }

  const refreshPermissions = async () => {
    try {
      setPermissions(await invoke<PermissionStatus>('check_permissions'))
    }
    catch { /* ignore */ }
  }

  let pollTimer: ReturnType<typeof setInterval> | undefined
  let unlisten: UnlistenFn | undefined
  let unlistenCloseChoice: UnlistenFn | undefined

  onMount(async () => {
    try {
      const initialConfig = await invoke<ReporterConfig>('get_config')
      if (initialConfig)
        setConfig(initialConfig)
      setStatus(await invoke<ConnectionStatus>('get_status'))
      await refreshPermissions()
    }
    catch (e) {
      console.error('Initialization error:', e)
    }

    pollTimer = setInterval(async () => {
      try {
        setStatus(await invoke<ConnectionStatus>('get_status'))
        await refreshPermissions()
      }
      catch { /* ignore */ }
    }, 1000)

    unlisten = await listen<any>('reporter-event', (event) => {
      const p = event.payload
      if (p.Log) {
        addLog(p.Log.level, p.Log.message)
      }
      else if (p.WindowUpdated) {
        const iconSrc = bytesToDataUri(p.WindowUpdated.icon_data)
        setWindowInfo({
          title: p.WindowUpdated.title,
          process_name: p.WindowUpdated.process_name,
          pid: p.WindowUpdated.pid,
          iconSrc,
        })
      }
      else if (p.MediaUpdated) {
        const artworkSrc = bytesToDataUri(p.MediaUpdated.artwork_data)
        setMediaInfo({
          title: p.MediaUpdated.title,
          artist: p.MediaUpdated.artist,
          album: p.MediaUpdated.album,
          duration: p.MediaUpdated.duration,
          elapsed_time: p.MediaUpdated.elapsed_time,
          playing: p.MediaUpdated.playing,
          artworkSrc,
        })
      }
    })

    unlistenCloseChoice = await listen('close-behavior-requested', () => {
      setRememberCloseChoice(true)
      setShowCloseChoice(true)
    })
  })

  onCleanup(() => {
    if (pollTimer !== undefined)
      clearInterval(pollTimer)
    unlisten?.()
    unlistenCloseChoice?.()
  })

  const toggleMonitor = async () => {
    if (status().is_monitoring) {
      await invoke('stop_monitor')
      setStatus({ is_monitoring: false, is_reporting: false, is_connected: false, last_error: null })
      setWindowInfo(null)
      setMediaInfo(null)
    }
    else {
      try {
        await invoke('start_monitor', { config: config() })
        setStatus({ ...status(), is_monitoring: true, last_error: null })
      }
      catch (e: any) {
        addLog('Error', `启动监听失败: ${e}`)
      }
    }
  }

  const toggleReporter = async () => {
    if (status().is_reporting) {
      await invoke('stop_reporter')
      setStatus({ ...status(), is_reporting: false, is_connected: false, last_error: null })
    }
    else {
      if (!config().ws_url || !config().token) {
        addLog('Error', '配置无效：请填写 WebSocket 地址和 Token')
        return
      }
      try {
        await invoke('save_config', { config: config() })
        await invoke('start_reporter', { config: config() })
        setStatus({ ...status(), is_reporting: true, is_connected: false, last_error: null })
      }
      catch (e: any) {
        addLog('Error', `启动上报失败: ${e}`)
      }
    }
  }

  const saveConfig = async () => {
    try {
      await invoke('save_config', { config: config() })
      addLog('Info', '配置已保存')
    }
    catch (e: any) {
      addLog('Error', `保存配置失败: ${e}`)
    }
  }

  const requestAccessibility = async () => {
    await invoke('request_permissions')
    await refreshPermissions()
  }

  const applyCloseBehavior = async (behavior: CloseBehavior) => {
    setShowCloseChoice(false)
    try {
      await invoke('apply_close_decision', {
        behavior,
        remember: rememberCloseChoice(),
      })
    }
    catch (e: any) {
      addLog('Error', `处理关闭行为失败: ${e}`)
    }
  }

  return (
    <div id="root">
      <Show when={showCloseChoice()}>
        <div class="modal-backdrop">
          <div class="modal-panel" role="dialog" aria-modal="true" aria-labelledby="close-choice-title">
            <div class="modal-header">
              <h2 id="close-choice-title">关闭窗口时如何处理？</h2>
              <p>选择后，本次关闭会立即执行；勾选后以后会直接按这个选择处理。</p>
            </div>

            <label class="checkbox-label close-choice-remember">
              <input
                type="checkbox"
                checked={rememberCloseChoice()}
                onChange={e => setRememberCloseChoice(e.currentTarget.checked)}
              />
              <span>记住我的选择</span>
            </label>

            <div class="modal-actions">
              <button class="btn btn-secondary modal-button" onClick={() => applyCloseBehavior('quit')}>
                直接退出
              </button>
              <button class="btn btn-primary modal-button" onClick={() => applyCloseBehavior('hide_to_tray')}>
                隐藏到托盘
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* 顶部统一状态与导航栏 */}
      <header class="app-header">
        <div class="brand">
          <img class="brand-icon" src={appIconUrl} alt="" />
          <h1>ShikenMatrix</h1>
          <span class="version">v0.1.0</span>
        </div>

        <div class="status-pills">
          <div class="pill">
            <Show when={status().is_monitoring} fallback={<IconMonitorOff class="pill-icon" />}>
              <IconMonitorOn class="pill-icon success" />
            </Show>
            <span>
              监听
              {status().is_monitoring ? '已启动' : '未启动'}
            </span>
          </div>
          <div class="pill">
            <Show when={status().is_connected} fallback={<IconWifiOff class="pill-icon" />}>
              <IconWifiOn class="pill-icon success" />
            </Show>
            <span>
              API
              {status().is_connected ? '已连接' : (status().is_reporting ? '连接中' : '未连接')}
            </span>
          </div>

          <Show
            when={!permissions().accessibility}
            fallback={(
              <div class="pill">
                <IconShieldOn class="pill-icon success" />
                <span>辅助功能(已授权)</span>
              </div>
            )}
          >
            <button class="pill clickable" onClick={requestAccessibility} title="点击请求辅助功能权限">
              <IconShieldOff class="pill-icon danger" />
              <span>辅助功能(未授权)</span>
            </button>
          </Show>

          <div class="pill">
            <Show when={permissions().media} fallback={<IconMusic class="pill-icon danger" />}>
              <IconMusic class="pill-icon success" />
            </Show>
            <span>媒体控制</span>
          </div>
        </div>
      </header>

      <main class="app-content">
        {/* 左侧功能区：配置与实时数据 */}
        <aside class="side-panel">

          <section>
            <h2 class="section-title">基础配置</h2>
            <div class="form-group">
              <label>WebSocket 地址</label>
              <input
                class="form-input"
                type="text"
                value={config().ws_url}
                onInput={e => setConfig({ ...config(), ws_url: e.currentTarget.value })}
              />
            </div>
            <div class="form-group">
              <label>认证 Token</label>
              <input
                class="form-input"
                type="password"
                value={config().token}
                onInput={e => setConfig({ ...config(), token: e.currentTarget.value })}
              />
            </div>
            <label class="checkbox-label">
              <input
                type="checkbox"
                checked={config().enable_media_reporting}
                onChange={e => setConfig({ ...config(), enable_media_reporting: e.currentTarget.checked })}
              />
              <span>上报媒体播放信息</span>
            </label>

            <div class="btn-group">
              <button class={`btn ${status().is_monitoring ? 'btn-danger' : 'btn-primary'}`} onClick={toggleMonitor}>
                <Show when={status().is_monitoring} fallback={<IconPlay />}>
                  <IconStop />
                </Show>
                {status().is_monitoring ? '停止监听' : '启动监听'}
              </button>

              <button
                class={`btn ${status().is_reporting ? 'btn-danger' : 'btn-success'}`}
                onClick={toggleReporter}
                disabled={!status().is_monitoring}
                title={!status().is_monitoring ? '需先启动监听' : ''}
              >
                <IconUpload />
                {status().is_reporting ? '停止上报' : '启动上报'}
              </button>

              <button class="btn btn-secondary" onClick={saveConfig}>
                <IconSave />
                保存
              </button>
            </div>
            <Show when={status().last_error}>
              <div class="error-msg">{status().last_error}</div>
            </Show>
          </section>

          <section class="info-section">
            <h2 class="section-title">实时状态</h2>

            <div class="info-card">
              <Show
                when={windowInfo()}
                fallback={(
                  <div class="empty-text">
                    <IconInbox class="empty-icon" />
                    <span>暂无前台窗口数据</span>
                  </div>
                )}
              >
                {win => (
                  <>
                    <Show when={win().iconSrc} fallback={<div class="info-icon" />}>
                      {src => <img class="info-icon" src={src()} alt="icon" />}
                    </Show>
                    <div class="info-details">
                      <div class="info-type">
                        <IconComputer class="info-type-icon" />
                        当前窗口
                      </div>
                      <div class="info-title">{win().title || '未知窗口'}</div>
                      <div class="info-sub">
                        {win().process_name}
                        {' '}
                        · PID
                        {' '}
                        {win().pid}
                      </div>
                    </div>
                  </>
                )}
              </Show>
            </div>

            <div class="info-card">
              <Show
                when={mediaInfo()}
                fallback={(
                  <div class="empty-text">
                    <IconInbox class="empty-icon" />
                    <span>暂无媒体播放数据</span>
                  </div>
                )}
              >
                {media => (
                  <>
                    <Show when={media().artworkSrc} fallback={<div class="info-icon" />}>
                      {src => <img class="info-icon" src={src()} alt="artwork" />}
                    </Show>
                    <div class="info-details">
                      <div class="info-type">
                        <IconMusic class="info-type-icon" />
                        媒体播放
                      </div>
                      <div class="info-title">{media().title || '未知媒体'}</div>
                      <div class="info-sub">
                        {media().artist}
                        {' '}
                        ·
                        {media().playing ? '播放中' : '已暂停'}
                        {' '}
                        /
                        {formatDuration(media().elapsed_time)}
                      </div>
                    </div>
                  </>
                )}
              </Show>
            </div>

          </section>
        </aside>

        {/* 右侧主视口：运行日志 */}
        <section class="log-panel">
          <div class="log-header">
            <span class="log-title">运行日志</span>
            <div class="log-actions">
              <div class="search-wrapper">
                <IconSearch class="search-icon" />
                <input
                  placeholder="搜索日志..."
                  value={searchText()}
                  onInput={e => setSearchText(e.currentTarget.value)}
                />
              </div>
              <button class="btn-clear" onClick={() => setLogs([])}>
                <IconDelete />
                清空
              </button>
            </div>
          </div>

          <div class="log-container">
            <Show
              when={filteredLogs().length > 0}
              fallback={(
                <div class="empty-logs">
                  <IconInbox class="empty-icon-lg" />
                  <span>监听暂未启动，无日志输出</span>
                </div>
              )}
            >
              <For each={filteredLogs()}>
                {entry => (
                  <div class="log-entry">
                    <span class="log-time">{entry.time}</span>
                    <span class={`log-level level-${entry.level.toLowerCase()}`}>{entry.level}</span>
                    <span class="log-message">{entry.message}</span>
                  </div>
                )}
              </For>
            </Show>
          </div>
        </section>

      </main>
    </div>
  )
}
