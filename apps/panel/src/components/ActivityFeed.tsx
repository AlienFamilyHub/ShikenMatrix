import type { ActivityEntry } from "../types";
import { formatClock } from "../lib/api";
import { For, Show } from "solid-js";
import IconActivity from "~icons/mingcute/align-justify-line";

export function ActivityFeed(props: { activity: ActivityEntry[]; now: number }) {
  return (
    <div class="glass-panel flex flex-col h-[500px]">
      <div class="p-6 border-b border-slate-200/50">
        <h3 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
          <IconActivity class="text-emerald-500 text-xl" />
          Activity Log
        </h3>
      </div>
      <div class="p-4 overflow-y-auto flex-1 flex flex-col gap-2">
        <Show
          when={props.activity.length > 0}
          fallback={<div class="text-center text-slate-400 py-8 font-medium">No recent activity</div>}
        >
          <For each={props.activity}>
            {item => (
              <div class="bg-white/60 p-3 rounded-lg border border-slate-100 flex gap-3 items-start text-sm hover:bg-white/80 transition-colors">
                <span class="text-slate-400 font-mono text-xs whitespace-nowrap mt-0.5">{formatClock(item.ts)}</span>
                <div class="flex flex-col gap-0.5 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="font-semibold text-slate-700">{item.summary}</span>
                    <Show when={item.client}>
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase bg-slate-100 text-slate-500 border border-slate-200">
                        {item.client}
                      </span>
                    </Show>
                  </div>
                  <Show when={item.detail}>
                    <span class="text-slate-500 truncate" title={item.detail!}>{item.detail}</span>
                  </Show>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
