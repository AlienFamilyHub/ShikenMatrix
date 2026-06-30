import type { DashboardSnapshot } from "../types";
import { formatUptime, formatClock } from "../lib/api";
import IconTime from "~icons/mingcute/time-line";
import IconServer from "~icons/mingcute/server-line";
import IconMessage from "~icons/mingcute/message-3-line";
import IconCalendar from "~icons/mingcute/calendar-month-line";

export function StatusOverview(props: { snapshot: DashboardSnapshot; now: number }) {
  return (
    <div class="glass-panel p-6">
      <h3 class="text-lg font-semibold text-slate-800 mb-5 flex items-center gap-2">
        <IconServer class="text-brand-500 text-xl" />
        System Status
      </h3>
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="flex items-center gap-1.5 text-xs uppercase tracking-wider font-semibold text-slate-400">
            <IconCalendar class="text-sm" /> Started At
          </div>
          <div class="text-2xl font-bold text-slate-700">{formatClock(props.snapshot.started_at)}</div>
        </div>
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="flex items-center gap-1.5 text-xs uppercase tracking-wider font-semibold text-slate-400">
            <IconTime class="text-sm" /> Uptime
          </div>
          <div class="text-2xl font-bold text-brand-500">{formatUptime(props.snapshot.uptime_seconds)}</div>
        </div>
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="flex items-center gap-1.5 text-xs uppercase tracking-wider font-semibold text-slate-400">
            <IconMessage class="text-sm" /> Total Messages
          </div>
          <div class="text-2xl font-bold text-slate-700">{props.snapshot.stats.total_messages}</div>
        </div>
        <div class="bg-white/60 p-4 rounded-xl border border-slate-100 shadow-sm flex flex-col gap-1">
          <div class="flex items-center gap-1.5 text-xs uppercase tracking-wider font-semibold text-slate-400">
            <IconServer class="text-sm" /> Bind Address
          </div>
          <div class="text-lg font-mono font-medium text-slate-600 mt-1">{props.snapshot.bind_addr}</div>
        </div>
      </div>
    </div>
  );
}
