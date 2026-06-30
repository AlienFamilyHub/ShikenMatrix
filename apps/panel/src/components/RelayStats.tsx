import type { DashboardSnapshot } from "../types";
import { formatRelative } from "../lib/api";
import IconUpload from "~icons/mingcute/upload-2-line";

export function RelayStats(props: { snapshot: DashboardSnapshot; now: number }) {
  return (
    <div class="glass-panel p-6">
      <h3 class="text-lg font-semibold text-slate-800 mb-5 flex items-center gap-2">
        <IconUpload class="text-indigo-500 text-xl" />
        Relay Statistics
      </h3>
      <div class="grid grid-cols-2 lg:grid-cols-5 gap-4">
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="text-xs uppercase tracking-wider font-semibold text-slate-400">Window Info</div>
          <div class="text-2xl font-bold text-slate-700">{props.snapshot.stats.window_info_count}</div>
        </div>
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="text-xs uppercase tracking-wider font-semibold text-slate-400">Media Playback</div>
          <div class="text-2xl font-bold text-slate-700">{props.snapshot.stats.media_playback_count}</div>
        </div>
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="text-xs uppercase tracking-wider font-semibold text-slate-400">Artwork Uploads</div>
          <div class="text-2xl font-bold text-slate-700">{props.snapshot.stats.artwork_uploads}</div>
        </div>
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="text-xs uppercase tracking-wider font-semibold text-slate-400">Upstream Errors</div>
          <div class="text-2xl font-bold text-red-500">{props.snapshot.stats.upstream_errors}</div>
        </div>
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="text-xs uppercase tracking-wider font-semibold text-slate-400">Last Activity</div>
          <div class="text-lg font-bold text-indigo-500 mt-1">{formatRelative(props.snapshot.stats.last_activity_at, props.now)}</div>
        </div>
      </div>
    </div>
  );
}
