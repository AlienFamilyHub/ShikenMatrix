import type { ConnectionStatus, ReporterConfig } from "../types";
import { Show } from "solid-js";
import IconPlay from "~icons/mingcute/play-circle-line";
import IconSave from "~icons/mingcute/save-2-line";
import IconStop from "~icons/mingcute/stop-circle-line";
import IconUpload from "~icons/mingcute/upload-2-line";

interface RunControlsProps {
  config: ReporterConfig;
  status: ConnectionStatus;
  onToggleMonitor: () => void;
  onToggleReporter: () => void;
  onSaveConfig: () => void;
}

export function RunControls(props: RunControlsProps) {
  return (
    <section>
      <h2 class="section-title">运行控制</h2>
      <div class="protocol-summary">
        <span>当前方案</span>
        <strong>{props.config.protocol === "native" ? "Native" : "Mix-Space"}</strong>
      </div>

      <label class="checkbox-label">
        <input
          type="checkbox"
          checked={props.config.enable_media_reporting}
          disabled
        />
        <span>
          媒体播放信息
          {props.config.enable_media_reporting ? "已启用" : "未启用"}
        </span>
      </label>

      <div class="btn-group">
        <button class={`btn ${props.status.is_monitoring ? "btn-danger" : "btn-primary"}`} onClick={() => props.onToggleMonitor()}>
          <Show when={props.status.is_monitoring} fallback={<IconPlay />}>
            <IconStop />
          </Show>
          {props.status.is_monitoring ? "停止监听" : "启动监听"}
        </button>

        <button
          class={`btn ${props.status.is_reporting ? "btn-danger" : "btn-success"}`}
          onClick={() => props.onToggleReporter()}
          disabled={!props.status.is_monitoring}
          title={!props.status.is_monitoring ? "需先启动监听" : ""}
        >
          <IconUpload />
          {props.status.is_reporting ? "停止上报" : "启动上报"}
        </button>

        <button class="btn btn-secondary" onClick={() => props.onSaveConfig()}>
          <IconSave />
          保存
        </button>
      </div>

      <Show when={props.status.last_error}>
        <div class="error-msg">{props.status.last_error}</div>
      </Show>
    </section>
  );
}
