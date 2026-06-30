import { createSignal, onCleanup, onMount, Show } from "solid-js";
import type { DashboardSnapshot, ClientKeyEntry } from "../types";
import { fetchSnapshot, saveUpstreamSettings, fetchClientKeys, createClientKey, deleteClientKey } from "../lib/api";
import { PanelHeader } from "../components/PanelHeader";
import { StatusOverview } from "../components/StatusOverview";
import { RelayStats } from "../components/RelayStats";
import { ClientList } from "../components/ClientList";
import { ActivityFeed } from "../components/ActivityFeed";
import { UpstreamSettingsForm } from "../components/UpstreamSettingsForm";
import Login from "./Login";
import IconKey from "~icons/mingcute/key-2-line";

const POLL_INTERVAL = 2000;

export default function Admin() {
  const initialAdminToken = localStorage.getItem("shikenmatrix_jwt") ?? "";
  const [token, setToken] = createSignal(initialAdminToken);
  const [snapshot, setSnapshot] = createSignal<DashboardSnapshot | null>(null);
  const [keys, setKeys] = createSignal<ClientKeyEntry[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [paused, setPaused] = createSignal(false);
  const [now, setNow] = createSignal(Date.now());
  const [saving, setSaving] = createSignal(false);
  const [saveError, setSaveError] = createSignal<string | null>(null);

  let abort: AbortController | undefined;
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let clockTimer: ReturnType<typeof setInterval> | undefined;

  const handleLogin = (newToken: string) => {
    localStorage.setItem("shikenmatrix_jwt", newToken);
    setToken(newToken);
    queueMicrotask(poll);
    loadClientKeys(newToken);
  };

  const handleLogout = () => {
    localStorage.removeItem("shikenmatrix_jwt");
    setToken("");
    setSnapshot(null);
  };

  const poll = async () => {
    if (paused() || !token()) return;
    abort?.abort();
    abort = new AbortController();
    try {
      const data = await fetchSnapshot(token(), abort.signal);
      setSnapshot(data);
      setError(null);
    } catch (err: any) {
      if (err instanceof DOMException && err.name === "AbortError") return;
      if (err.message.includes("401")) {
        handleLogout();
      } else {
        setError(err.message);
      }
    } finally {
      setLoading(false);
    }
  };

  const loadClientKeys = async (t = token()) => {
    if (!t) return;
    try {
      const k = await fetchClientKeys(t);
      setKeys(k);
    } catch(e) {}
  };

  const handleCreateKey = async (desc: string) => {
    if (!desc || !token()) return;
    try {
      const { api_key } = await createClientKey(token(), desc);
      alert(`Client Key Created: ${api_key}\nPlease copy it now.`);
      loadClientKeys();
    } catch(e: any) {
      alert(`Failed to create key: ${e.message}`);
    }
  };

  const handleDeleteKey = async (id: number) => {
    if (!token() || !confirm("Delete this client key?")) return;
    try {
      await deleteClientKey(token(), id);
      loadClientKeys();
    } catch(e: any) {
      alert(`Failed to delete key: ${e.message}`);
    }
  }

  onMount(() => {
    if (token()) {
      poll();
      loadClientKeys();
    }
    pollTimer = setInterval(poll, POLL_INTERVAL);
    clockTimer = setInterval(() => setNow(Date.now()), 1000);
  });

  onCleanup(() => {
    abort?.abort();
    if (pollTimer) clearInterval(pollTimer);
    if (clockTimer) clearInterval(clockTimer);
  });

  return (
    <Show when={token()} fallback={<Login onLogin={handleLogin} />}>
      <div class="min-h-screen flex flex-col">
        <PanelHeader
          snapshot={snapshot()}
          loading={loading()}
          error={error()}
          paused={paused()}
          now={now()}
          onTogglePause={() => setPaused(!paused())}
          onRefresh={poll}
          onLogout={handleLogout}
        />
        <main class="flex-1 max-w-7xl w-full mx-auto p-4 sm:p-6 lg:p-8 flex flex-col gap-6">
          <Show when={snapshot()} fallback={
            <div class="flex items-center justify-center h-64 text-slate-500 font-medium animate-pulse text-lg">
              Loading Dashboard...
            </div>
          }>
            {snap => (
              <>
                <StatusOverview snapshot={snap()} now={now()} />
                <RelayStats snapshot={snap()} now={now()} />

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
                  <div class="flex flex-col gap-6">
                    <div class="glass-panel p-6 flex flex-col gap-6">
                      <h3 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
                        <IconKey class="text-amber-500 text-xl" />
                        Client API Keys
                      </h3>
                      <div class="flex flex-col gap-3 max-h-64 overflow-y-auto pr-2">
                        {keys().map(k => (
                          <div class="flex items-center justify-between p-3 bg-white/60 rounded-xl border border-slate-100 shadow-sm gap-4 hover:shadow-md transition-shadow">
                            <div class="flex flex-col min-w-0 flex-1">
                              <span class="font-bold text-slate-700 truncate">{k.description}</span>
                              <span class="font-mono text-xs text-slate-400 bg-slate-50 px-2 py-0.5 rounded-md mt-1 truncate">{k.api_key}</span>
                            </div>
                            <button class="px-3 py-1.5 text-xs font-bold text-red-600 bg-red-50 hover:bg-red-100 rounded-lg transition-colors" onClick={() => handleDeleteKey(k.id)}>Revoke</button>
                          </div>
                        ))}
                        {keys().length === 0 && <div class="text-sm text-slate-400 text-center py-4 bg-slate-50/50 rounded-xl border border-dashed border-slate-200">No keys generated yet.</div>}
                      </div>
                      <form onSubmit={(e) => {
                        e.preventDefault();
                        const input = (e.currentTarget.elements.namedItem("desc") as HTMLInputElement);
                        handleCreateKey(input.value);
                        input.value = "";
                      }} class="flex gap-2">
                        <input name="desc" class="input-modern flex-1 text-sm" placeholder="e.g. desktop-app" required />
                        <button type="submit" class="btn-primary text-sm whitespace-nowrap">Create Key</button>
                      </form>
                    </div>

                    <UpstreamSettingsForm
                      settings={snap().upstream}
                      saving={saving()}
                      error={saveError()}
                      onSave={async (s) => {
                        setSaving(true);
                        setSaveError(null);
                        try {
                          const data = await saveUpstreamSettings(s, token());
                          setSnapshot(data);
                        } catch(e: any) {
                          setSaveError(e.message);
                        } finally {
                          setSaving(false);
                        }
                      }}
                    />
                  </div>

                  <div class="flex flex-col gap-6">
                    <ClientList clients={snap().clients} now={now()} />
                    <ActivityFeed activity={snap().activity} now={now()} />
                  </div>
                </div>
              </>
            )}
          </Show>
        </main>
      </div>
    </Show>
  );
}
