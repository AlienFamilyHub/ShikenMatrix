<script setup lang="ts" vapor>
import type { RouteName } from "@/lib/router";
import IconDevices from "~icons/mingcute/device-line";
import IconMoon from "~icons/mingcute/moon-line";
import IconSliders from "~icons/mingcute/settings-3-line";
import IconSun from "~icons/mingcute/sun-line";
import IconButton from "@/components/ui/icon-button.vue";
import ToastHost from "@/components/ui/toast-host.vue";
import { hrefFor, navigate, route } from "@/lib/router";
import { theme, toggleTheme } from "@/lib/theme";
import ControlView from "@/views/control-view.vue";
import PresenceView from "@/views/presence-view.vue";

function followLink(next: RouteName) {
  return (event: MouseEvent) => {
    if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
      return;
    }
    event.preventDefault();
    navigate(next);
  };
}
</script>

<template>
  <div class="shell">
    <a class="skip-link" href="#main">跳到主要内容</a>

    <header class="topbar">
      <div class="topbar__inner">
        <a class="brand" :href="hrefFor('presence')" aria-label="Presence 首页" @click="followLink('presence')">
          <span class="brand__mark">
            <IconDevices />
          </span>
          <span class="brand__text">Presence</span>
        </a>

        <nav class="nav" aria-label="主导航">
          <a
            class="nav__link" :href="hrefFor('presence')" :aria-current="route === 'presence' ? 'page' : undefined"
            @click="followLink('presence')"
          >
            <IconDevices />
            <span>状态</span>
          </a>
          <a
            class="nav__link" :href="hrefFor('control')" :aria-current="route === 'control' ? 'page' : undefined"
            @click="followLink('control')"
          >
            <IconSliders />
            <span>控制</span>
          </a>
        </nav>

        <div class="topbar__actions">
          <IconButton
            :label="theme === 'dark' ? '切换到浅色主题' : '切换到深色主题'" :title="theme === 'dark' ? '浅色主题' : '深色主题'"
            @click="toggleTheme"
          >
            <IconSun v-if="theme === 'dark'" />
            <IconMoon v-else />
          </IconButton>
        </div>
      </div>
    </header>

    <main id="main" class="page">
      <ControlView v-if="route === 'control'" />
      <PresenceView v-else />
    </main>

    <ToastHost />
  </div>
</template>

<style>
/*
 * Global layer: design tokens plus the document-wide reset. Custom properties
 * cannot live in a scoped block (scope attributes stop `:root` from matching),
 * so tokens stay here while every component owns its own rules.
 */
:root {
  color-scheme: light;

  --font-sans:
    -apple-system, BlinkMacSystemFont, "Segoe UI Variable Text", "Segoe UI", "PingFang SC", "Hiragino Sans GB",
    "Microsoft YaHei", "Noto Sans CJK SC", system-ui, sans-serif;
  --font-mono: ui-monospace, "SF Mono", "JetBrains Mono", Menlo, Consolas, monospace;

  --bg: #f3f3f8;
  --bg-glow: radial-gradient(90rem 40rem at 105% -10%, #e6e0ff 0%, transparent 55%);
  --surface: #ffffff;
  --surface-muted: #f7f7fb;
  --surface-strong: #eeeef5;
  --border: #e4e4ee;
  --border-strong: #d3d3e0;

  --text: #16151d;
  --muted: #5f5c70;
  --faint: #8a8799;

  --accent: #6c5ce7;
  --accent-strong: #5747d6;
  --accent-weak: #9689f5;
  --on-accent: #ffffff;
  --accent-soft: color-mix(in oklab, var(--accent) 13%, transparent);

  --success: #107a54;
  --success-soft: color-mix(in oklab, var(--success) 13%, transparent);
  --warning: #925a06;
  --warning-soft: color-mix(in oklab, var(--warning) 15%, transparent);
  --danger: #bd2b21;
  --danger-soft: color-mix(in oklab, var(--danger) 11%, transparent);

  --shadow-1: 0 1px 2px rgb(22 21 29 / 0.04), 0 6px 18px -12px rgb(22 21 29 / 0.22);
  --shadow-2: 0 32px 64px -32px rgb(22 21 29 / 0.4);

  --r-sm: 10px;
  --r-md: 14px;
  --r-lg: 20px;
  --r-full: 999px;

  --ease: cubic-bezier(0.2, 0.8, 0.2, 1);
  --tap: 44px;
}

html[data-theme="dark"] {
  color-scheme: dark;

  --bg: #111013;
  --bg-glow: radial-gradient(90rem 40rem at 105% -10%, #2a2340 0%, transparent 55%);
  --surface: #1a1922;
  --surface-muted: #201f2a;
  --surface-strong: #272633;
  --border: #2d2c39;
  --border-strong: #3b3a4a;

  --text: #f1eff8;
  --muted: #a8a4ba;
  --faint: #807d92;

  --accent: #9c8dff;
  --accent-strong: #b0a4ff;
  --accent-weak: #6f61c9;
  --on-accent: #14121f;
  --accent-soft: color-mix(in oklab, var(--accent) 18%, transparent);

  --success: #4fd1a1;
  --success-soft: color-mix(in oklab, var(--success) 16%, transparent);
  --warning: #efc069;
  --warning-soft: color-mix(in oklab, var(--warning) 16%, transparent);
  --danger: #ff8b80;
  --danger-soft: color-mix(in oklab, var(--danger) 15%, transparent);

  --shadow-1: 0 1px 2px rgb(0 0 0 / 0.4), 0 8px 24px -16px rgb(0 0 0 / 0.7);
  --shadow-2: 0 32px 64px -32px rgb(0 0 0 / 0.8);
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

* {
  margin: 0;
}

html {
  /* Keep the scrollbar gutter reserved: live data changes the page height, and
     a scrollbar appearing or disappearing would shift every element sideways. */
  scrollbar-gutter: stable;
  -webkit-text-size-adjust: 100%;
}

body {
  min-height: 100dvh;
  background-color: var(--bg);
  background-image: var(--bg-glow);
  background-repeat: no-repeat;
  background-attachment: fixed;
  color: var(--text);
  font-family: var(--font-sans);
  font-size: 15px;
  line-height: 1.55;
  -webkit-font-smoothing: antialiased;
  text-rendering: optimizeLegibility;
}

h1,
h2,
h3,
h4 {
  font-weight: 620;
  letter-spacing: -0.01em;
  line-height: 1.25;
  text-wrap: balance;
}

p,
h1,
h2,
h3,
h4 {
  overflow-wrap: break-word;
}

img,
svg {
  display: block;
  max-width: 100%;
}

/* unplugin-icons emits bare <svg> roots; every icon is sized by its context. */
svg {
  width: 1.25em;
  height: 1.25em;
  flex: none;
}

button,
input,
textarea {
  font: inherit;
  color: inherit;
}

button {
  cursor: pointer;
  background: none;
  border: 0;
  padding: 0;
}

a {
  color: inherit;
  text-decoration: none;
}

ul,
ol {
  list-style: none;
  padding: 0;
}

code,
kbd {
  font-family: var(--font-mono);
  font-size: 0.92em;
}

:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

::selection {
  background: var(--accent-soft);
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
</style>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  min-height: 100dvh;
}

.skip-link {
  position: absolute;
  left: 50%;
  top: 0.5rem;
  translate: -50% -160%;
  z-index: 60;
  padding: 0.6rem 1rem;
  border-radius: var(--r-full);
  background: var(--accent);
  color: var(--on-accent);
  font-weight: 600;
  transition: translate 160ms var(--ease);
}

.skip-link:focus-visible {
  translate: -50% 0;
}

.topbar {
  position: sticky;
  top: 0;
  z-index: 40;
  border-bottom: 1px solid color-mix(in oklab, var(--border) 72%, transparent);
  background: color-mix(in oklab, var(--bg) 84%, transparent);
  backdrop-filter: blur(16px) saturate(150%);
}

.topbar__inner {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: min(1120px, 100%);
  margin-inline: auto;
  padding: 0.65rem clamp(1rem, 4vw, 1.75rem);
}

.brand {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  font-size: 15.5px;
  font-weight: 650;
  letter-spacing: -0.01em;
}

.brand__mark {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border-radius: 10px;
  background: linear-gradient(140deg, var(--accent), var(--accent-weak));
  color: var(--on-accent);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.28);
}

.brand__mark svg {
  width: 1.15em;
  height: 1.15em;
}

.brand__text {
  display: none;
}

@media (min-width: 480px) {
  .brand__text {
    display: inline;
  }
}

.nav {
  display: flex;
  gap: 2px;
  margin-left: auto;
  padding: 3px;
  border-radius: var(--r-full);
  background: var(--surface-strong);
}

.nav__link {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-height: 34px;
  padding: 0 0.75rem;
  border-radius: var(--r-full);
  color: var(--muted);
  font-size: 13.5px;
  font-weight: 560;
  transition:
    background-color 150ms var(--ease),
    color 150ms var(--ease);
}

.nav__link:hover {
  color: var(--text);
}

.nav__link[aria-current="page"] {
  background: var(--surface);
  color: var(--text);
  box-shadow: var(--shadow-1);
}

.nav__link svg {
  width: 1.1em;
  height: 1.1em;
}

.topbar__actions {
  display: flex;
  align-items: center;
  gap: 0.1rem;
}

.page {
  flex: 1;
  width: min(1120px, 100%);
  margin-inline: auto;
  padding: clamp(1.25rem, 3vw, 2rem) clamp(1rem, 4vw, 1.75rem) 4rem;
}

@media (max-width: 560px) {
  .nav__link span {
    display: none;
  }

  .nav__link {
    padding: 0 0.7rem;
  }
}
</style>
