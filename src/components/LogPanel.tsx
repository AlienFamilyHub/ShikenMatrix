import type { LogEntry } from "../types";
import { For, Show } from "solid-js";
import IconDelete from "~icons/mingcute/delete-2-line";
import IconInbox from "~icons/mingcute/inbox-line";
import IconSearch from "~icons/mingcute/search-2-line";

interface LogPanelProps {
  logs: LogEntry[];
  searchText: string;
  onSearchTextChange: (text: string) => void;
  onClear: () => void;
}

export function LogPanel(props: LogPanelProps) {
  const filteredLogs = () => {
    const search = props.searchText.toLowerCase().trim();
    if (!search)
      return props.logs;
    return props.logs.filter(log => log.message.toLowerCase().includes(search));
  };

  return (
    <section class="log-panel">
      <div class="log-header">
        <span class="log-title">运行日志</span>
        <div class="log-actions">
          <div class="search-wrapper">
            <IconSearch class="search-icon" />
            <input
              placeholder="搜索日志..."
              value={props.searchText}
              onInput={event => props.onSearchTextChange(event.currentTarget.value)}
            />
          </div>
          <button class="btn-clear" onClick={() => props.onClear()}>
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
  );
}
