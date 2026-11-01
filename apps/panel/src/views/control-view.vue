<script setup lang="ts" vapor>
import type { AdminSnapshot } from "@/lib/types";
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import IconLogout from "~icons/mingcute/exit-line";
import IconKey from "~icons/mingcute/key-1-line";
import IconLan from "~icons/mingcute/link-line";
import IconLock from "~icons/mingcute/lock-line";
import IconSliders from "~icons/mingcute/settings-3-line";
import IconData from "~icons/mingcute/storage-line";
import AccessSection from "@/components/control/access-section.vue";
import DataSection from "@/components/control/data-section.vue";
import KeysSection from "@/components/control/keys-section.vue";
import LoginView from "@/components/control/login-view.vue";
import SecuritySection from "@/components/control/security-section.vue";
import UpstreamSection from "@/components/control/upstream-section.vue";
import AppButton from "@/components/ui/app-button.vue";
import PageHeading from "@/components/ui/page-heading.vue";
import StateNote from "@/components/ui/state-note.vue";
import { api, clearToken, getToken } from "@/lib/api";
import { formatCount, formatUnixTime, formatUptime } from "@/lib/format";

type AuthState = "checking" | "anonymous" | "authed";

const SECTIONS = [
  { id: "upstream", label: "上游转发" },
  { id: "access", label: "接入控制" },
  { id: "keys", label: "客户端密钥" },
  { id: "security", label: "账户安全" },
  { id: "data", label: "数据管理" },
];

const auth = ref<AuthState>("checking");
const snapshot = ref<AdminSnapshot | null>(null);
const activeSection = ref("upstream");

let observer: IntersectionObserver | undefined;

function observeSections() {
  observer?.disconnect();
  if (!("IntersectionObserver" in window))
    return;
  const targets = SECTIONS.map(section => document.getElementById(section.id)).filter(
    (element): element is HTMLElement => element !== null,
  );
  if (targets.length === 0)
    return;
  observer = new IntersectionObserver(
    (entries) => {
      const visible = entries
        .filter(entry => entry.isIntersecting)
        .sort((left, right) => right.intersectionRatio - left.intersectionRatio)[0];
      if (visible)
        activeSection.value = visible.target.id;
    },
    {
      rootMargin: "-88px 0px -55% 0px",
      threshold: [0, 0.2, 1],
    },
  );
  for (const target of targets) observer.observe(target);
}

async function loadSnapshot() {
  try {
    snapshot.value = await api.getState();
  } catch {
    snapshot.value = null;
  }
}

async function verify() {
  if (!getToken()) {
    auth.value = "anonymous";
    return;
  }
  try {
    snapshot.value = await api.getState();
    auth.value = "authed";
  } catch {
    auth.value = "anonymous";
  }
}

function onUnauthorized() {
  auth.value = "anonymous";
  snapshot.value = null;
}

function onLoggedIn() {
  auth.value = "authed";
  void loadSnapshot();
}

function logout() {
  clearToken();
  auth.value = "anonymous";
  snapshot.value = null;
}

watch(auth, async (state) => {
  if (state !== "authed")
    return;
  await nextTick();
  observeSections();
});

onMounted(() => {
  window.addEventListener("shikenmatrix:unauthorized", onUnauthorized);
  void verify();
});

onUnmounted(() => {
  window.removeEventListener("shikenmatrix:unauthorized", onUnauthorized);
  observer?.disconnect();
});

const upstreamLabel = computed(() => {
  const config = snapshot.value?.config;
  if (!config)
    return "—";
  if (!config.upstream_enabled)
    return "已关闭";
  return config.upstream_protocol === "mix_space"
    ? "Mix-Space"
    : "Native";
});

const mediaLabel = computed(() => {
  const config = snapshot.value?.config;
  if (!config)
    return "—";
  return config.media_reporting_enabled ? "开启" : "关闭";
});
</script>

<template>
  <StateNote v-if="auth === 'checking'">
    正在验证登录状态…
  </StateNote>

  <LoginView v-else-if="auth === 'anonymous'" @success="onLoggedIn" />

  <div v-else class="control-main">
    <PageHeading
      eyebrow="Control"
      title="控制面板"
      description="管理上报转发、客户端接入、密钥与账户安全。"
    >
      <template #icon>
        <IconSliders />
      </template>
      <template #aside>
        <AppButton size="sm" @click="logout">
          <IconLogout />退出登录
        </AppButton>
      </template>
    </PageHeading>

    <div class="overview">
      <div class="overview__item">
        <span class="overview__label">服务运行</span>
        <span class="overview__value">
          {{ snapshot ? formatUptime(snapshot.uptime_seconds) : '—' }}
        </span>
      </div>
      <div class="overview__item">
        <span class="overview__label">监听地址</span>
        <span class="overview__value">{{ snapshot?.bind_addr ?? '—' }}</span>
      </div>
      <div class="overview__item">
        <span class="overview__label">在线客户端</span>
        <span class="overview__value">
          {{ snapshot ? `${formatCount(snapshot.clients.length)} 个` : '—' }}
        </span>
      </div>
      <div class="overview__item">
        <span class="overview__label">上游协议</span>
        <span class="overview__value">{{ upstreamLabel }}</span>
      </div>
      <div class="overview__item">
        <span class="overview__label">媒体上报</span>
        <span class="overview__value">{{ mediaLabel }}</span>
      </div>
      <div class="overview__item">
        <span class="overview__label">最近活动</span>
        <span class="overview__value">
          {{ formatUnixTime(snapshot?.stats.last_activity_at ?? null) ?? '暂无' }}
        </span>
      </div>
    </div>

    <div class="control-grid">
      <nav class="control-nav" aria-label="设置分区">
        <a
          v-for="section in SECTIONS"
          :key="section.id"
          class="control-nav__link"
          :href="`#${section.id}`"
          :aria-current="activeSection === section.id ? 'true' : undefined"
          @click="activeSection = section.id"
        >
          <IconLan v-if="section.id === 'access'" />
          <IconKey v-else-if="section.id === 'keys'" />
          <IconLock v-else-if="section.id === 'security'" />
          <IconData v-else-if="section.id === 'data'" />
          <IconSliders v-else />
          {{ section.label }}
        </a>
      </nav>

      <div class="control-main">
        <UpstreamSection />
        <AccessSection />
        <KeysSection />
        <SecuritySection />
        <DataSection />
      </div>
    </div>
  </div>
</template>

<style scoped>
.control-main {
  display: grid;
  gap: 1rem;
  min-width: 0;
}

.control-grid {
  display: grid;
  gap: 1rem;
  align-items: start;
}

@media (min-width: 980px) {
  .control-grid {
    grid-template-columns: 208px minmax(0, 1fr);
  }
}

.control-nav {
  display: flex;
  gap: 0.15rem;
  padding: 0.4rem;
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  background: var(--surface);
  box-shadow: var(--shadow-1);
  scrollbar-width: none;
}

.control-nav::-webkit-scrollbar {
  display: none;
}

@media (min-width: 980px) {
  .control-nav {
    position: sticky;
    top: 5.5rem;
    flex-direction: column;
    overflow: visible;
  }
}

.control-nav__link {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 38px;
  padding: 0 0.6rem;
  border-radius: var(--r-sm);
  color: var(--muted);
  font-size: 13.5px;
  font-weight: 545;
  white-space: nowrap;
  transition:
    background-color 150ms var(--ease),
    color 150ms var(--ease);
}

.control-nav__link:hover {
  background: var(--surface-muted);
  color: var(--text);
}

.control-nav__link[aria-current="true"] {
  background: var(--accent-soft);
  color: var(--accent-strong);
}

.control-nav__link svg {
  width: 1.15em;
  height: 1.15em;
}

.overview {
  display: grid;
  gap: 0.6rem;
  grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
}

.overview__item {
  display: grid;
  gap: 0.1rem;
  padding: 0.7rem 0.85rem;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface);
}

.overview__label {
  color: var(--faint);
  font-size: 11.5px;
}

.overview__value {
  font-size: 15px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
</style>
