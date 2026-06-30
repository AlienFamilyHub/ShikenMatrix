import type { PublicSnapshot } from "../types";
import { createSignal, onCleanup, onMount, Show } from "solid-js";
import IconWindow from "~icons/mingcute/computer-line";
import IconMusic from "~icons/mingcute/music-2-line";
import IconTime from "~icons/mingcute/time-line";
import { fetchPublicSnapshot, formatRelative } from "../lib/api";

const POLL_INTERVAL = 2000;

export default function Share() {
  const [snapshot, setSnapshot] = createSignal<PublicSnapshot | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [now, setNow] = createSignal(Date.now());

  let abort: AbortController | undefined;
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let clockTimer: ReturnType<typeof setInterval> | undefined;

  const poll = async () => {
    abort?.abort();
    abort = new AbortController();
    try {
      const data = await fetchPublicSnapshot(abort.signal);
      setSnapshot(data);
      setError(null);
    } catch (err: unknown) {
      if (err instanceof DOMException && err.name === "AbortError")
        return;
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  onMount(() => {
    poll();
    pollTimer = setInterval(poll, POLL_INTERVAL);
    clockTimer = setInterval(() => setNow(Date.now()), 1000);
  });

  onCleanup(() => {
    abort?.abort();
    if (pollTimer)
      clearInterval(pollTimer);
    if (clockTimer)
      clearInterval(clockTimer);
  });

  return (
    <div class="min-h-screen flex items-center justify-center p-4">
      <Show
        when={snapshot()}
        fallback={<div class="animate-pulse text-slate-500 font-medium text-lg">{error() ? `Connection failed: ${error()}` : "Connecting to ShikenMatrix..."}</div>}
      >
        {snap => (
          <div class="glass-panel max-w-md w-full p-8 md:p-10 flex flex-col gap-8 rounded-3xl">
            <div class="text-center">
              <h1 class="text-2xl font-bold bg-linear-to-r from-brand-500 to-indigo-500 bg-clip-text text-transparent">Blogger Status</h1>
            </div>

            <div class="flex flex-col gap-6">
              <div class="flex items-start gap-4 group">
                <div class="p-3 bg-blue-50 text-brand-500 rounded-xl group-hover:scale-110 transition-transform">
                  <IconWindow class="text-xl" />
                </div>
                <div class="flex flex-col gap-1 flex-1">
                  <span class="text-xs uppercase tracking-wider font-semibold text-slate-400">Current Window</span>
                  <span class="text-slate-800 font-medium leading-snug">{snap().current_window || "—"}</span>
                </div>
              </div>

              <div class="flex items-start gap-4 group">
                <div class="p-3 bg-purple-50 text-purple-500 rounded-xl group-hover:scale-110 transition-transform">
                  <IconMusic class="text-xl" />
                </div>
                <div class="flex flex-col gap-1 flex-1">
                  <span class="text-xs uppercase tracking-wider font-semibold text-slate-400">Now Playing</span>
                  <span class="text-slate-800 font-medium leading-snug">{snap().current_media || "—"}</span>
                </div>
              </div>

              <div class="flex items-start gap-4 group">
                <div class="p-3 bg-emerald-50 text-emerald-500 rounded-xl group-hover:scale-110 transition-transform">
                  <IconTime class="text-xl" />
                </div>
                <div class="flex flex-col gap-1 flex-1">
                  <span class="text-xs uppercase tracking-wider font-semibold text-slate-400">Last Active</span>
                  <span class="text-slate-800 font-medium leading-snug">{formatRelative(snap().last_activity_at, now())}</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
}
