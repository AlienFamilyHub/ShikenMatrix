import type { UnlistenFn } from '@tauri-apps/api/event'
import type {
  AppPage,
  CloseBehavior,
  ConnectionStatus,
  LogEntry,
  MediaView,
  PermissionStatus,
  ReporterConfig,
  WindowView,
} from './types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { createSignal, Match, onCleanup, onMount, Show, Switch } from 'solid-js'
import { AppHeader } from './components/AppHeader'
import { CloseChoiceModal } from './components/CloseChoiceModal'
import { bytesToDataUri, normalizeReporterConfig, nowLabel } from './lib/format'
import { AboutPage } from './pages/AboutPage'
import { MonitorPage } from './pages/MonitorPage'
import { SettingsPage } from './pages/SettingsPage'
import './App.css'

const MAX_LOGS = 200

export default function App() {
  const [page, setPage] = createSignal<AppPage>('monitor')
  const [config, setConfig] = createSignal<ReporterConfig>(normalizeReporterConfig({}))
  const [status, setStatus] = createSignal<ConnectionStatus>({ is_monitoring: false, is_reporting: false, is_connected: false, last_error: null })
  const [permissions, setPermissions] = createSignal<PermissionStatus>({ accessibility: false, media: false })
  const [logs, setLogs] = createSignal<LogEntry[]>([])
  const [searchText, setSearchText] = createSignal('')
  const [windowInfo, setWindowInfo] = createSignal<WindowView | null>(null)
  const [mediaInfo, setMediaInfo] = createSignal<MediaView | null>(null)
  const [showCloseChoice, setShowCloseChoice] = createSignal(false)
  const [rememberCloseChoice, setRememberCloseChoice] = createSignal(true)

  let pollTimer: ReturnType<typeof setInterval> | undefined
  let unlistenReporter: UnlistenFn | undefined
  let unlistenCloseChoice: UnlistenFn | undefined
  let unlistenNavigate: UnlistenFn | undefined

  const addLog = (level: string, message: string) => {
    setLogs((previousLogs) => {
      const entry: LogEntry = { time: nowLabel(), level: level as LogEntry['level'], message }
      const nextLogs = [...previousLogs, entry]
      return nextLogs.length > MAX_LOGS ? nextLogs.slice(nextLogs.length - MAX_LOGS) : nextLogs
    })
  }

  const refreshPermissions = async () => {
    try {
      setPermissions(await invoke<PermissionStatus>('check_permissions'))
    }
    catch { /* ignore transient permission query failures */ }
  }

  const refreshRuntimeStatus = async () => {
    try {
      setStatus(await invoke<ConnectionStatus>('get_status'))
      await refreshPermissions()
    }
    catch { /* ignore transient polling failures */ }
  }

  const handleReporterEvent = (payload: any) => {
    if (payload.Log) {
      addLog(payload.Log.level, payload.Log.message)
    }
    else if (payload.WindowUpdated) {
      setWindowInfo({
        title: payload.WindowUpdated.title,
        process_name: payload.WindowUpdated.process_name,
        pid: payload.WindowUpdated.pid,
        iconSrc: bytesToDataUri(payload.WindowUpdated.icon_data),
      })
    }
    else if (payload.MediaUpdated) {
      setMediaInfo({
        title: payload.MediaUpdated.title,
        artist: payload.MediaUpdated.artist,
        album: payload.MediaUpdated.album,
        duration: payload.MediaUpdated.duration,
        elapsed_time: payload.MediaUpdated.elapsed_time,
        playing: payload.MediaUpdated.playing,
        artworkSrc: bytesToDataUri(payload.MediaUpdated.artwork_data),
      })
    }
  }

  const loadInitialState = async () => {
    try {
      const initialConfig = await invoke<Partial<ReporterConfig>>('get_config')
      setConfig(normalizeReporterConfig(initialConfig ?? {}))
      await refreshRuntimeStatus()
    }
    catch (error) {
      console.error('Initialization error:', error)
    }
  }

  onMount(async () => {
    await loadInitialState()
    pollTimer = setInterval(refreshRuntimeStatus, 1000)
    unlistenReporter = await listen('reporter-event', event => handleReporterEvent(event.payload))
    unlistenCloseChoice = await listen('close-behavior-requested', () => {
      setRememberCloseChoice(true)
      setShowCloseChoice(true)
    })
    unlistenNavigate = await listen<AppPage>('navigate', (event) => {
      setPage(event.payload)
    })
  })

  onCleanup(() => {
    if (pollTimer !== undefined)
      clearInterval(pollTimer)
    unlistenReporter?.()
    unlistenCloseChoice?.()
    unlistenNavigate?.()
  })

  const saveConfig = async () => {
    try {
      await invoke('save_config', { config: config() })
      addLog('Info', '配置已保存')
    }
    catch (error: any) {
      addLog('Error', `保存配置失败: ${error}`)
      throw error
    }
  }

  const toggleMonitor = async () => {
    if (status().is_monitoring) {
      await invoke('stop_monitor')
      setStatus({ is_monitoring: false, is_reporting: false, is_connected: false, last_error: null })
      setWindowInfo(null)
      setMediaInfo(null)
      return
    }

    try {
      await invoke('start_monitor', { config: config() })
      setStatus({ ...status(), is_monitoring: true, last_error: null })
    }
    catch (error: any) {
      addLog('Error', `启动监听失败: ${error}`)
    }
  }

  const toggleReporter = async () => {
    if (status().is_reporting) {
      await invoke('stop_reporter')
      setStatus({ ...status(), is_reporting: false, is_connected: false, last_error: null })
      return
    }

    if (!isReporterConfigValid(config())) {
      addLog('Error', '配置无效：请检查当前上报方案的必填字段')
      setPage('settings')
      return
    }

    try {
      await saveConfig()
      await invoke('start_reporter', { config: config() })
      setStatus({ ...status(), is_reporting: true, is_connected: false, last_error: null })
    }
    catch (error: any) {
      addLog('Error', `启动上报失败: ${error}`)
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
    catch (error: any) {
      addLog('Error', `处理关闭行为失败: ${error}`)
    }
  }

  return (
    <div id="root">
      <Show when={showCloseChoice()}>
        <CloseChoiceModal
          remember={rememberCloseChoice()}
          onRememberChange={setRememberCloseChoice}
          onApply={applyCloseBehavior}
        />
      </Show>

      <AppHeader
        page={page()}
        status={status()}
        permissions={permissions()}
        onPageChange={setPage}
        onRequestAccessibility={requestAccessibility}
      />

      <Switch>
        <Match when={page() === 'monitor'}>
          <MonitorPage
            config={config()}
            status={status()}
            logs={logs()}
            searchText={searchText()}
            windowInfo={windowInfo()}
            mediaInfo={mediaInfo()}
            onSearchTextChange={setSearchText}
            onClearLogs={() => setLogs([])}
            onToggleMonitor={toggleMonitor}
            onToggleReporter={toggleReporter}
            onSaveConfig={saveConfig}
          />
        </Match>
        <Match when={page() === 'settings'}>
          <SettingsPage config={config()} onConfigChange={setConfig} onSave={saveConfig} />
        </Match>
        <Match when={page() === 'about'}>
          <AboutPage />
        </Match>
      </Switch>
    </div>
  )
}

function isReporterConfigValid(config: ReporterConfig): boolean {
  if (config.protocol === 'native')
    return config.native.ws_url.trim().length > 0 && config.native.token.trim().length > 0

  if (config.mix_space.endpoint.trim().length === 0 || config.mix_space.token.trim().length === 0)
    return false

  if (!config.s3.enabled)
    return true

  return config.s3.bucket.trim().length > 0
    && config.s3.region.trim().length > 0
    && config.s3.access_key.trim().length > 0
    && config.s3.secret_key.trim().length > 0
}
