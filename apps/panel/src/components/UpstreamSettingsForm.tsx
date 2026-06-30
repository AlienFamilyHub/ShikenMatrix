import type { UpstreamSettings } from "../types";
import { createEffect, createSignal, Show } from "solid-js";
import IconSettings from "~icons/mingcute/settings-2-line";

export function UpstreamSettingsForm(props: {
  settings: UpstreamSettings;
  saving: boolean;
  error: string | null;
  onSave: (s: UpstreamSettings) => Promise<void>;
}) {
  const [local, setLocal] = createSignal(props.settings);
  createEffect(() => setLocal(props.settings));

  const update = <K extends keyof UpstreamSettings>(k: K, v: UpstreamSettings[K]) => {
    setLocal(prev => ({ ...prev, [k]: v }));
  };

  return (
    <div class="glass-panel p-6 flex flex-col gap-6">
      <h3 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
        <IconSettings class="text-slate-500 text-xl" />
        Upstream Configuration
      </h3>
      <form class="flex flex-col gap-5" onSubmit={e => { e.preventDefault(); props.onSave(local()); }}>
        
        <div class="flex flex-col gap-1.5">
          <span class="text-sm font-semibold text-slate-700">Protocol</span>
          <select class="input-modern" value={local().protocol} onChange={e => update("protocol", e.currentTarget.value as any)}>
            <option value="native">Native WebSocket</option>
            <option value="mix_space">Mix-Space</option>
          </select>
        </div>

        <label class="flex items-center gap-3 p-3 bg-white/50 rounded-lg border border-slate-200 cursor-pointer hover:bg-white/80 transition-colors">
          <input type="checkbox" class="w-4 h-4 text-brand-500 rounded border-slate-300 focus:ring-brand-500" checked={local().enable_media_reporting} onChange={e => update("enable_media_reporting", e.currentTarget.checked)} />
          <span class="text-sm font-medium text-slate-700">Enable Media Reporting</span>
        </label>

        <Show when={local().protocol === "native"}>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4 bg-blue-50/50 rounded-xl border border-blue-100/50">
            <div class="flex flex-col gap-1.5 md:col-span-2">
              <span class="text-sm font-semibold text-slate-700">WebSocket URL</span>
              <input class="input-modern" type="text" value={local().native_ws_url} onInput={e => update("native_ws_url", e.currentTarget.value)} placeholder="ws://..." />
            </div>
            <div class="flex flex-col gap-1.5 md:col-span-2">
              <span class="text-sm font-semibold text-slate-700">Access Token</span>
              <input class="input-modern" type="password" value={local().native_token} onInput={e => update("native_token", e.currentTarget.value)} />
            </div>
          </div>
        </Show>

        <Show when={local().protocol === "mix_space"}>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4 bg-purple-50/50 rounded-xl border border-purple-100/50">
            <div class="flex flex-col gap-1.5 md:col-span-2">
              <span class="text-sm font-semibold text-slate-700">API Endpoint</span>
              <input class="input-modern" type="text" value={local().mix_space_endpoint} onInput={e => update("mix_space_endpoint", e.currentTarget.value)} placeholder="https://api..." />
            </div>
          </div>
        </Show>

        <Show when={props.error}>
          <div class="p-3 bg-red-50 text-red-600 text-sm rounded-lg font-medium">{props.error}</div>
        </Show>

        <div class="pt-2">
          <button type="submit" class="btn-primary w-full md:w-auto" disabled={props.saving}>
            {props.saving ? "Saving..." : "Save Configuration"}
          </button>
        </div>
      </form>
    </div>
  );
}
