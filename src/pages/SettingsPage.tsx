import type { ReporterConfig } from "../types";
import gsap from "gsap";
import { createSignal, onMount, Show } from "solid-js";
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

  const updateConfig = (patch: Partial<ReporterConfig>) => {
    props.onConfigChange({ ...props.config, ...patch });
    setHasChanges(true);
  };
  const updateNative = (patch: Partial<ReporterConfig["native"]>) => {
    props.onConfigChange({
      ...props.config,
      native: { ...props.config.native, ...patch },
    });
    setHasChanges(true);
  };
  const updateMixSpace = (patch: Partial<ReporterConfig["mix_space"]>) => {
    props.onConfigChange({
      ...props.config,
      mix_space: { ...props.config.mix_space, ...patch },
    });
    setHasChanges(true);
  };
  const updateS3 = (patch: Partial<ReporterConfig["s3"]>) => {
    props.onConfigChange({
      ...props.config,
      s3: { ...props.config.s3, ...patch },
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
            <h2>上报方案</h2>
            <button
              class="btn btn-primary save-settings-button"
              onClick={handleSave}
              disabled={!hasChanges()}
            >
              <IconSave />
              保存设置
            </button>
          </div>

          <div class="segmented-control" role="tablist" aria-label="上报方案">
            <button
              class={
                props.config.protocol === "native"
                  ? "segment active"
                  : "segment"
              }
              onClick={() => updateConfig({ protocol: "native" })}
            >
              Native
            </button>
            <button
              class={
                props.config.protocol === "mix_space"
                  ? "segment active"
                  : "segment"
              }
              onClick={() => updateConfig({ protocol: "mix_space" })}
            >
              Mix-Space
            </button>
          </div>

          <label class="checkbox-label settings-checkbox">
            <input
              type="checkbox"
              checked={props.config.enable_media_reporting}
              onChange={event =>
                updateConfig({
                  enable_media_reporting: event.currentTarget.checked,
                })}
            />
            <span>上报媒体播放信息</span>
          </label>
        </section>

        <Show
          when={props.config.protocol === "native"}
          fallback={(
            <MixSpaceFields
              config={props.config}
              onUpdate={updateMixSpace}
              onS3Update={updateS3}
            />
          )}
        >
          <NativeFields config={props.config} onUpdate={updateNative} />
        </Show>
      </div>
    </main>
  );
}

interface ProtocolFieldsProps {
  config: ReporterConfig;
}

interface NativeFieldsProps extends ProtocolFieldsProps {
  onUpdate: (patch: Partial<ReporterConfig["native"]>) => void;
}

function NativeFields(props: NativeFieldsProps) {
  let sectionRef!: HTMLElement;

  onMount(() => {
    gsap.from(sectionRef, {
      opacity: 0,
      y: 6,
      duration: 0.3,
      ease: "power2.out",
    });
  });

  return (
    <section class="settings-section" ref={sectionRef}>
      <h2>Native 协议</h2>
      <p class="settings-note">
        Native 使用 WebSocket 长连接，窗口与媒体事件分开实时上报。
      </p>
      <FormInput
        label="WebSocket 地址"
        value={props.config.native.ws_url}
        placeholder="wss://example.com/reporter"
        onInput={value => props.onUpdate({ ws_url: value })}
      />
      <FormInput
        label="认证 Token"
        value={props.config.native.token}
        type="password"
        onInput={value => props.onUpdate({ token: value })}
      />
    </section>
  );
}

interface MixSpaceFieldsProps extends ProtocolFieldsProps {
  onUpdate: (patch: Partial<ReporterConfig["mix_space"]>) => void;
  onS3Update: (patch: Partial<ReporterConfig["s3"]>) => void;
}

function MixSpaceFields(props: MixSpaceFieldsProps) {
  let containerRef!: HTMLDivElement;

  onMount(() => {
    gsap.from(containerRef.children, {
      opacity: 0,
      y: 6,
      duration: 0.3,
      stagger: 0.06,
      ease: "power2.out",
    });
  });

  return (
    <div
      ref={containerRef}
      style={{ "display": "flex", "flex-direction": "column", "gap": "24px" }}
    >
      <section class="settings-section">
        <h2>Mix-Space 协议</h2>
        <p class="settings-note">
          Mix-Space 使用上游 ProcessReporter 的 HTTP JSON
          结构，将窗口与媒体合并成一次上报。
        </p>
        <FormInput
          label="Endpoint"
          value={props.config.mix_space.endpoint}
          placeholder="https://example.com/api/process"
          onInput={value => props.onUpdate({ endpoint: value })}
        />
        <FormInput
          label="API Token"
          value={props.config.mix_space.token}
          type="password"
          onInput={value => props.onUpdate({ token: value })}
        />
        <label class="form-group">
          <span>请求方法</span>
          <select
            class="form-input"
            value={props.config.mix_space.method}
            onChange={event =>
              props.onUpdate({ method: event.currentTarget.value })}
          >
            <option value="POST">POST</option>
            <option value="PUT">PUT</option>
            <option value="PATCH">PATCH</option>
          </select>
        </label>
      </section>

      <S3Fields config={props.config} onUpdate={props.onS3Update} />
    </div>
  );
}

interface S3FieldsProps extends ProtocolFieldsProps {
  onUpdate: (patch: Partial<ReporterConfig["s3"]>) => void;
}

function S3Fields(props: S3FieldsProps) {
  return (
    <section class="settings-section">
      <h2>S3 Icons</h2>
      <p class="settings-note">
        启用后会先上传应用图标和媒体封面，再分别写入 process.iconUrl 与
        media.icon。
      </p>
      <label class="checkbox-label settings-toggle-row">
        <input
          type="checkbox"
          checked={props.config.s3.enabled}
          onChange={event =>
            props.onUpdate({ enabled: event.currentTarget.checked })}
        />
        <span>启用 S3 图标上传</span>
      </label>
      <div class="settings-grid">
        <FormInput
          label="Bucket"
          value={props.config.s3.bucket}
          onInput={value => props.onUpdate({ bucket: value })}
        />
        <FormInput
          label="Region"
          value={props.config.s3.region}
          onInput={value => props.onUpdate({ region: value })}
        />
        <FormInput
          label="Access Key"
          value={props.config.s3.access_key}
          onInput={value => props.onUpdate({ access_key: value })}
        />
        <FormInput
          label="Secret Key"
          value={props.config.s3.secret_key}
          type="password"
          onInput={value => props.onUpdate({ secret_key: value })}
        />
      </div>
      <FormInput
        label="对象路径模板"
        value={props.config.s3.key_template}
        placeholder="{kind}/{Y}/{M}/{D}/{SHA}.{ext}"
        onInput={value => props.onUpdate({ key_template: value })}
      />
      <div class="variable-help">
        <span>
          {"{kind}"}
          : app-icons / media-icons
        </span>
        <span>
          {"{Y}"}
          : 年
        </span>
        <span>
          {"{M}"}
          : 月
        </span>
        <span>
          {"{D}"}
          : 日
        </span>
        <span>
          {"{SHA}"}
          : 文件 SHA-256
        </span>
        <span>
          {"{ext}"}
          : 扩展名
        </span>
        <span>
          {"{APP}"}
          : 应用名
        </span>
      </div>
      <FormInput
        label="生命周期天数"
        value={String(props.config.s3.lifecycle_days)}
        type="number"
        placeholder="0"
        onInput={value =>
          props.onUpdate({ lifecycle_days: Number(value) || 0 })}
      />
      <FormInput
        label="Endpoint"
        value={props.config.s3.endpoint}
        placeholder="https://account.r2.cloudflarestorage.com"
        onInput={value => props.onUpdate({ endpoint: value })}
      />
      <FormInput
        label="Custom Domain"
        value={props.config.s3.custom_domain}
        placeholder="https://assets.example.com"
        onInput={value => props.onUpdate({ custom_domain: value })}
      />
    </section>
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
