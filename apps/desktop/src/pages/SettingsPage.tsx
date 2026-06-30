import type { ReporterConfig } from "../types";
import gsap from "gsap";
import { createSignal, onMount } from "solid-js";
import IconSave from "~icons/mingcute/save-2-line";

interface SettingsPageProps {
  config: ReporterConfig;
  onConfigChange: (config: ReporterConfig) => void;
  onSave: () => void;
}

export function SettingsPage(props: SettingsPageProps) {
  let shellRef!: HTMLDivElement;
  const [hasChanges, setHasChanges] = createSignal(false);

  onMount(() => {
    gsap.from(shellRef.children, {
      y: 6,
      opacity: 0,
      duration: 0.35,
      stagger: 0.06,
      ease: "power2.out",
    });
  });

  const updateServer = (patch: Partial<ReporterConfig["server"]>) => {
    props.onConfigChange({
      ...props.config,
      server: { ...props.config.server, ...patch },
    });
    setHasChanges(true);
  };

  const handleSave = async () => {
    await props.onSave();
    setHasChanges(false);
  };

  return (
    <main class="settings-page">
      <div class="settings-shell" ref={shellRef}>
        <section class="settings-section">
          <div class="settings-section-header">
            <h2>Server 上报</h2>
            <button
              class="btn btn-primary save-settings-button"
              onClick={handleSave}
              disabled={!hasChanges()}
            >
              <IconSave />
              保存设置
            </button>
          </div>

          <p class="settings-note">
            Desktop 只连接 ShikenMatrix Server。上游、鉴权、媒体转发和 S3 配置统一在 Server Admin 中管理。
          </p>

          <FormInput
            label="Server WebSocket 地址"
            value={props.config.server.ws_url}
            placeholder="ws://127.0.0.1:4317/reporter"
            onInput={value => updateServer({ ws_url: value })}
          />

        </section>
      </div>
    </main>
  );
}

interface FormInputProps {
  label: string;
  value: string;
  placeholder?: string;
  type?: string;
  onInput: (value: string) => void;
}

function FormInput(props: FormInputProps) {
  return (
    <label class="form-group">
      <span>{props.label}</span>
      <input
        class="form-input"
        type={props.type ?? "text"}
        value={props.value}
        placeholder={props.placeholder}
        onInput={event => props.onInput(event.currentTarget.value)}
      />
    </label>
  );
}
