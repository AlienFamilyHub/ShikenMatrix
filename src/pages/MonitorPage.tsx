import type { ConnectionStatus, LogEntry, MediaView, ReporterConfig, WindowView } from "../types";
import { LogPanel } from "../components/LogPanel";
import { RunControls } from "../components/RunControls";
import { RuntimeCards } from "../components/RuntimeCards";

interface MonitorPageProps {
  config: ReporterConfig;
  status: ConnectionStatus;
  logs: LogEntry[];
  searchText: string;
  windowInfo: WindowView | null;
  mediaInfo: MediaView | null;
  onSearchTextChange: (text: string) => void;
  onClearLogs: () => void;
  onToggleMonitor: () => void;
  onToggleReporter: () => void;
  onSaveConfig: () => void;
}

export function MonitorPage(props: MonitorPageProps) {
  return (
    <main class="app-content">
      <aside class="side-panel">
        <RunControls
          config={props.config}
          status={props.status}
          onToggleMonitor={props.onToggleMonitor}
          onToggleReporter={props.onToggleReporter}
          onSaveConfig={props.onSaveConfig}
        />
        <RuntimeCards windowInfo={props.windowInfo} mediaInfo={props.mediaInfo} />
      </aside>

      <LogPanel
        logs={props.logs}
        searchText={props.searchText}
        onSearchTextChange={props.onSearchTextChange}
        onClear={props.onClearLogs}
      />
    </main>
  );
}
