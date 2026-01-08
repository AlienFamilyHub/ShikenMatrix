<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import ForegroundWidget from "@/components/ForegroundWidget.vue";
import MediaWidget from "@/components/MediaWidget.vue";

// Types matching Rust structs
interface WindowInfo {
	title: string;
	process_name: string;
	icon_data?: number[];
	app_id?: string;
	pid: number;
}

interface MediaMetadata {
	bundle_identifier?: string;
	title?: string;
	artist?: string;
	album?: string;
	duration: number;
	artwork_data?: string;
	artwork_mime_type?: string;
	content_item_identifier?: string;
}

interface PlaybackState {
	playing: boolean;
	playback_rate: number;
	elapsed_time: number;
}

// State
const foregroundInfo = ref<WindowInfo | null>(null);
const mediaMetadata = ref<MediaMetadata | null>(null);
const playbackState = ref<PlaybackState | null>(null);

let pollingInterval: number | null = null;

const loading = ref(true);
const errorMsg = ref<string | null>(null);
const permissionGranted = ref(false);
const containerRef = ref<HTMLElement | null>(null);

// 窗口尺寸配置
const MIN_WIDTH = 320; // 最小宽度
const MAX_WIDTH = 450; // 最大宽度
const DRAG_HANDLE_HEIGHT = 36; // 拖拽区域高度

// 计算是否显示媒体组件
const showMedia = computed(() => {
	if (!mediaMetadata.value)
		return false;
	const hasContent = !!(mediaMetadata.value.title || mediaMetadata.value.artist);
	const isPlaying = playbackState.value?.playing ?? false;
	return hasContent && isPlaying;
});

// 计算文字所需宽度
const measureTextWidth = (text: string, fontSize: number, fontWeight: number = 400): number => {
	const canvas = document.createElement("canvas");
	const ctx = canvas.getContext("2d");
	if (!ctx)
		return 0;
	ctx.font = `${fontWeight} ${fontSize}px -apple-system, BlinkMacSystemFont, "PingFang SC", sans-serif`;
	return ctx.measureText(text).width;
};

// 防抖变量
let updateWindowTimer: ReturnType<typeof setTimeout> | null = null;
let lastWindowSize = { width: -1, height: -1 };

// 动态调整窗口大小
const updateWindowSize = async (debounce: boolean = true) => {
	// 防抖处理，避免频繁调整
	if (debounce) {
		if (updateWindowTimer) {
			clearTimeout(updateWindowTimer);
		}
		updateWindowTimer = setTimeout(() => updateWindowSize(false), 100);
		return;
	}

	await nextTick();
	if (!containerRef.value)
		return;

	// 如果设置面板打开（主内容被模糊），不调整窗口大小
	const mainContent = document.querySelector(".main-content.blurred");
	if (mainContent) {
		return;
	}

	try {
		const window = getCurrentWindow();

		// 计算所需宽度（优先单行显示）
		let neededWidth = MIN_WIDTH;

		// 前台应用信息 - 卡片内边距 28px * 2 = 56px
		const cardPadding = 56;
		const containerPadding = 40; // 容器内边距 20px * 2

		if (foregroundInfo.value) {
			const appNameWidth = measureTextWidth(foregroundInfo.value.process_name, 20, 700);
			const titleWidth = measureTextWidth(foregroundInfo.value.title || "", 13, 400);
			const maxTextWidth = Math.max(appNameWidth, titleWidth);
			// 布局改为行布局：图标72px + gap20px + 卡片内边距56px + 容器内边距40px
			neededWidth = Math.max(neededWidth, maxTextWidth + 72 + 20 + cardPadding + containerPadding);
		}

		// 媒体信息 - 封面72px + gap20px + 卡片内边距24px*2
		if (showMedia.value && mediaMetadata.value) {
			const trackWidth = measureTextWidth(mediaMetadata.value.title || "", 16, 700);
			const artistWidth = measureTextWidth(mediaMetadata.value.artist || "", 13, 500);
			const albumWidth = measureTextWidth(mediaMetadata.value.album || "", 12, 400);
			const maxMediaTextWidth = Math.max(trackWidth, artistWidth, albumWidth);
			const mediaContentWidth = maxMediaTextWidth + 72 + 20 + 48 + containerPadding;
			neededWidth = Math.max(neededWidth, mediaContentWidth);
		}

		// 限制在最大宽度内
		const finalWidth = Math.min(Math.max(neededWidth, MIN_WIDTH), MAX_WIDTH);

		// 确保有最小高度，避免窗口太小看不到内容
		let contentHeight = 200; // 默认最小内容高度
		if (containerRef.value) {
			const rect = containerRef.value.getBoundingClientRect();
			contentHeight = Math.max(rect.height, 200); // 确保至少200px高度
		}
		const totalHeight = DRAG_HANDLE_HEIGHT + contentHeight; // 不再添加额外间距，避免累积增长

		// 向下取整，避免累积误差
		const newW = Math.floor(finalWidth);
		const newH = Math.floor(totalHeight);

		// 如果是第一次初始化或者尺寸发生明显变化，则调整窗口
		const isFirstTime = lastWindowSize.width === -1 || lastWindowSize.height === -1;
		const sizeThreshold = 3;
		if (isFirstTime
			|| Math.abs(lastWindowSize.width - newW) >= sizeThreshold || Math.abs(lastWindowSize.height - newH) >= sizeThreshold) {
			try {
				await window.setSize(new LogicalSize(newW, newH));
				lastWindowSize = { width: newW, height: newH };
			} catch (err) {
				console.error("设置窗口大小失败:", err);
			}
		}
	} catch (e) {
		console.error("调整窗口大小失败:", e);
	}
};

// 监听内容变化，调整窗口大小
watch([loading, errorMsg, foregroundInfo, showMedia], () => {
	updateWindowSize();
}, { flush: "post" });

// 检查并请求权限
const checkAndRequestPermission = async () => {
	try {
		console.warn("开始检查权限...");
		// 先检查权限
		const hasPermission = await invoke<boolean>("check_permissions");
		console.warn("权限状态:", hasPermission);
		if (hasPermission) {
			permissionGranted.value = true;
			return true;
		}

		console.warn("请求权限...");
		// 请求权限（会弹出系统对话框）
		const granted = await invoke<boolean>("request_permissions");
		console.warn("权限授予结果:", granted);
		permissionGranted.value = granted;
		return granted;
	} catch (error) {
		console.error("权限检查失败:", error);
		return false;
	}
};

// Data fetching
const fetchData = async () => {
	try {
		// Fetch foreground window info (后端会自动上报)
		const windowInfo = await invoke<WindowInfo>("get_frontmost_window");
		foregroundInfo.value = windowInfo;

		// Fetch media info
		const metadata = await invoke<MediaMetadata | null>("get_media_metadata_cmd");
		mediaMetadata.value = metadata;

		// Fetch playback state (后端会自动上报)
		const state = await invoke<PlaybackState | null>("get_playback_state_cmd");
		playbackState.value = state;

		errorMsg.value = null;
	} catch (error) {
		console.error("Error fetching data:", error);
		errorMsg.value = String(error);
	} finally {
		loading.value = false;
	}
};

onMounted(async () => {
	// 先请求权限
	const hasPermission = await checkAndRequestPermission();
	if (!hasPermission) {
		errorMsg.value = "需要辅助功能权限，请在系统设置中授权后重启应用";
		loading.value = false;
		return;
	}

	// 从 localStorage 加载设置并初始化上报
	try {
		const saved = localStorage.getItem("owner-desktop-settings");
		if (saved) {
			const settings = JSON.parse(saved);
			await invoke("update_reporter_config", {
				config: {
					enabled: settings.reportEnabled || false,
					ws_url: settings.wsUrl || "",
					token: settings.token || "",
				},
			});
		}
	} catch (error) {
		console.error("初始化上报配置失败:", error);
	}

	fetchData();
	// Poll every 1 second
	pollingInterval = window.setInterval(fetchData, 1000);
});

onUnmounted(() => {
	if (pollingInterval) {
		clearInterval(pollingInterval);
	}
	if (updateWindowTimer) {
		clearTimeout(updateWindowTimer);
		updateWindowTimer = null;
	}
});
</script>

<template>
	<div ref="containerRef" class="widgets-container">
		<!-- 加载状态 -->
		<Transition name="fade-scale">
			<div v-if="loading" class="state-card loading-card">
				<div class="state-icon-wrapper">
					<div class="loading-spinner" />
				</div>
				<p class="state-text">
					正在加载...
				</p>
			</div>
		</Transition>

		<!-- 错误状态 -->
		<Transition name="fade-scale">
			<div v-if="!loading && errorMsg" class="state-card error-card">
				<div class="state-icon-wrapper error">
					<Icon name="mdi:alert-circle-outline" class="state-icon" />
				</div>
				<p class="error-title">
					获取数据失败
				</p>
				<p class="error-msg">
					{{ errorMsg }}
				</p>
				<button class="retry-btn" @click="fetchData">
					重试
				</button>
			</div>
		</Transition>

		<!-- 正常显示 -->
		<Transition name="content-appear">
			<div v-if="!loading && !errorMsg" class="content-wrapper">
				<ForegroundWidget :info="foregroundInfo" />
				<MediaWidget :metadata="mediaMetadata" :playback-state="playbackState" />
			</div>
		</Transition>
	</div>
</template>

<style lang="scss" scoped>
$transition-spring: cubic-bezier(0.34, 1.56, 0.64, 1);

.widgets-container {
	display: flex;
	flex-direction: column;
	width: 100%;
	min-height: 100%;
	padding: 20px;
	box-sizing: border-box;
	gap: 20px;
}

.content-wrapper {
	display: flex;
	flex-direction: column;
	gap: 20px;
	width: 100%;
}

// 状态卡片通用样式
.state-card {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	padding: 40px 28px;
	background: var(--bg-glass);
	border: 1px solid var(--border-glass);
	border-radius: 28px;
	backdrop-filter: blur(24px);
	-webkit-backdrop-filter: blur(24px);
	text-align: center;
	gap: 16px;
}

.state-icon-wrapper {
	width: 64px;
	height: 64px;
	display: flex;
	align-items: center;
	justify-content: center;
	background: rgba(255, 255, 255, 0.1);
	border-radius: 50%;

	&.error {
		background: rgba(255, 59, 48, 0.1);

		.state-icon {
			color: #ff3b30;
		}
	}
}

.state-icon {
	font-size: 32px;
	color: var(--text-secondary);
}

.state-text {
	margin: 0;
	font-size: 16px;
	font-weight: 500;
	color: var(--text-secondary);
}

// 加载动画
.loading-spinner {
	width: 32px;
	height: 32px;
	border: 3px solid rgba(120, 120, 128, 0.2);
	border-top-color: var(--accent-color);
	border-radius: 50%;
	animation: spin 1s linear infinite;
}

@keyframes spin {
	to {
		transform: rotate(360deg);
	}
}

// 错误卡片
.error-title {
	margin: 0;
	font-size: 18px;
	font-weight: 700;
	color: var(--text-primary);
}

.error-msg {
	margin: 0;
	font-size: 14px;
	color: var(--text-secondary);
	word-break: break-word;
	max-width: 100%;
	line-height: 1.5;
}

.retry-btn {
	margin-top: 8px;
	padding: 12px 28px;
	background: var(--accent-color);
	color: white;
	border: none;
	border-radius: 100px;
	font-size: 15px;
	font-weight: 600;
	cursor: pointer;
	transition: all 0.3s $transition-spring;
	box-shadow: 0 4px 12px rgba(0, 122, 255, 0.3);

	&:hover {
		transform: translateY(-2px) scale(1.02);
		box-shadow: 0 6px 20px rgba(0, 122, 255, 0.4);
	}

	&:active {
		transform: translateY(0) scale(0.98);
	}
}

// 过渡动画
.fade-scale-enter-active,
.fade-scale-leave-active {
	transition: all 0.35s ease-out;
}

.fade-scale-enter-from,
.fade-scale-leave-to {
	opacity: 0;
	transform: scale(0.96);
}

.content-appear-enter-active {
	transition:
		opacity 0.4s ease-out,
		transform 0.5s ease-out;
}

.content-appear-leave-active {
	transition:
		opacity 0.25s ease-in,
		transform 0.25s ease-in;
}

.content-appear-enter-from {
	opacity: 0;
	transform: translateY(12px);
}

.content-appear-leave-to {
	opacity: 0;
	transform: translateY(-8px);
}
</style>
