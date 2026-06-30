import type { ClientEntry } from "../types";
import { formatRelative } from "../lib/api";
import { For, Show } from "solid-js";
import IconComputer from "~icons/mingcute/computer-line";
import IconPhone from "~icons/mingcute/cellphone-line";
import IconGroup from "~icons/mingcute/group-line";

export function ClientList(props: { clients: ClientEntry[]; now: number }) {
  return (
    <div class="glass-panel flex flex-col h-full max-h-[500px]">
      <div class="p-6 border-b border-slate-200/50">
        <h3 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
          <IconGroup class="text-blue-500 text-xl" />
          Active Clients
          <span class="ml-auto bg-blue-100 text-blue-700 py-0.5 px-2.5 rounded-full text-sm font-bold">{props.clients.length}</span>
        </h3>
      </div>
      <div class="p-4 overflow-y-auto flex-1 flex flex-col gap-3">
        <Show
          when={props.clients.length > 0}
          fallback={<div class="text-center text-slate-400 py-8 font-medium">No clients connected</div>}
        >
          <For each={props.clients}>
            {client => (
              <div class="bg-white/60 rounded-xl p-4 border border-slate-100 shadow-sm hover:shadow-md transition-shadow flex flex-col gap-3">
                <div class="flex items-center justify-between border-b border-slate-100 pb-2">
                  <div class="flex items-center gap-2">
                    <Show when={client.kind === "desktop_reporter"} fallback={<IconPhone class="text-purple-500 text-lg" />}>
                      <IconComputer class="text-blue-500 text-lg" />
                    </Show>
                    <span class="font-bold text-slate-700 capitalize">{client.kind.replace("_", " ")}</span>
                    <span class="text-xs font-mono text-slate-400 bg-slate-100 px-1.5 py-0.5 rounded">#{client.id}</span>
                  </div>
                  <span class="text-xs font-semibold text-emerald-500 bg-emerald-50 px-2 py-1 rounded-md">
                    Connected {formatRelative(client.connected_at, props.now)}
                  </span>
                </div>
                <div class="flex flex-col gap-1 text-sm">
                  <div class="flex items-start gap-2">
                    <span class="text-slate-400 font-medium min-w-[70px]">Client:</span>
                    <span class="text-slate-700 font-mono truncate">{client.client_info || "—"}</span>
                  </div>
                  <div class="flex items-start gap-2">
                    <span class="text-slate-400 font-medium min-w-[70px]">Window:</span>
                    <span class="text-slate-700 truncate">{client.last_window || "—"}</span>
                  </div>
                  <div class="flex items-start gap-2">
                    <span class="text-slate-400 font-medium min-w-[70px]">Media:</span>
                    <span class="text-slate-700 truncate">{client.last_media || "—"}</span>
                  </div>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
