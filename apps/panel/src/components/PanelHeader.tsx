import type { DashboardSnapshot } from "../types";
import { Show } from "solid-js";
import IconHeart from "~icons/mingcute/heart-line";
import IconPause from "~icons/mingcute/pause-line";
import IconRefresh from "~icons/mingcute/refresh-2-line";
import IconWifiOff from "~icons/mingcute/wifi-off-line";
import IconLogout from "~icons/mingcute/exit-line";
import appIconUrl from "../assets/icon.svg";

interface PanelHeaderProps {
  snapshot: DashboardSnapshot | null;
  loading: boolean;
  error: string | null;
  paused: boolean;
  now: number;
  onTogglePause: () => void;
  onRefresh: () => void;
  onLogout?: () => void;
}

export function PanelHeader(props: PanelHeaderProps) {
  const healthy = () => props.snapshot !== null && props.error === null;

  return (
    <header class="sticky top-0 z-50 bg-white/80 backdrop-blur-lg border-b border-slate-200 shadow-sm">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <img class="w-8 h-8" src={appIconUrl} alt="Logo" />
          <div>
            <h1 class="text-lg font-bold text-slate-800 leading-none tracking-tight">ShikenMatrix</h1>
            <span class="text-[10px] uppercase font-bold text-brand-500 tracking-widest">Admin Panel</span>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <div class={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider ${healthy() ? "bg-emerald-50 text-emerald-600" : "bg-red-50 text-red-600"}`}>
            <Show when={healthy()} fallback={<IconWifiOff class="text-sm" />}>
              <IconHeart class="text-sm" />
            </Show>
            <span>{props.loading ? "Loading..." : props.error ? "Offline" : "Healthy"}</span>
          </div>

          <div class="flex items-center gap-2 border-l border-slate-200 pl-4">
            <button
              class="p-2 text-slate-400 hover:text-brand-500 hover:bg-brand-50 rounded-lg transition-colors"
              onClick={() => props.onRefresh()}
              title="Refresh"
            >
              <IconRefresh class="text-lg" />
            </button>
            <button
              class={`p-2 rounded-lg transition-colors ${props.paused ? "text-amber-500 bg-amber-50" : "text-slate-400 hover:text-brand-500 hover:bg-brand-50"}`}
              onClick={() => props.onTogglePause()}
              title={props.paused ? "Resume auto-refresh" : "Pause auto-refresh"}
            >
              <IconPause class="text-lg" />
            </button>
            <Show when={props.onLogout}>
              <button
                class="ml-2 flex items-center gap-1.5 px-3 py-1.5 bg-red-50 hover:bg-red-100 text-red-600 rounded-lg text-sm font-semibold transition-colors"
                onClick={() => props.onLogout?.()}
                title="Logout"
              >
                <IconLogout class="text-sm" />
                <span class="hidden sm:inline">Logout</span>
              </button>
            </Show>
          </div>
        </div>
      </div>
    </header>
  );
}
