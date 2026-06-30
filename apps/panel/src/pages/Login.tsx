import { createSignal } from "solid-js";
import { login } from "../lib/api";
import IconLock from "~icons/mingcute/lock-line";

export default function Login(props: { onLogin: (token: string) => void }) {
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      const data = await login(username(), password());
      props.onLogin(data.token);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="min-h-screen flex items-center justify-center p-4">
      <div class="glass-panel max-w-sm w-full p-8 rounded-2xl flex flex-col gap-6">
        <div class="text-center flex flex-col items-center gap-2">
          <div class="h-12 w-12 bg-gradient-to-tr from-brand-500 to-indigo-500 rounded-2xl flex items-center justify-center text-white shadow-lg mb-2">
            <IconLock class="text-2xl" />
          </div>
          <h1 class="text-2xl font-bold text-slate-800 tracking-tight">Admin Login</h1>
          <p class="text-sm text-slate-500">Sign in to manage ShikenMatrix</p>
        </div>

        <form onSubmit={handleSubmit} class="flex flex-col gap-4">
          <label class="flex flex-col gap-1.5">
            <span class="text-sm font-medium text-slate-700">Username</span>
            <input
              type="text"
              required
              class="input-modern"
              value={username()}
              onInput={e => setUsername(e.currentTarget.value)}
            />
          </label>
          <label class="flex flex-col gap-1.5">
            <span class="text-sm font-medium text-slate-700">Password</span>
            <input
              type="password"
              required
              class="input-modern"
              value={password()}
              onInput={e => setPassword(e.currentTarget.value)}
            />
          </label>
          {error() && (
            <div class="p-3 bg-red-50 text-red-600 text-sm rounded-lg text-center font-medium">
              {error()}
            </div>
          )}
          <button type="submit" class="btn-primary mt-2" disabled={loading()}>
            {loading() ? "Authenticating..." : "Sign In"}
          </button>
        </form>
      </div>
    </div>
  );
}
