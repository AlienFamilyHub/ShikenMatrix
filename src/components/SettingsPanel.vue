<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import gsap from "gsap";
import { ref, watch } from "vue";

interface SettingsData {
	reportEnabled: boolean;
	wsUrl: string;
	token: string;
}

const props = defineProps<{
	show: boolean;
}>();

const emit = defineEmits<{
	close: [];
	save: [settings: SettingsData];
}>();

const settings = ref<SettingsData>({
	reportEnabled: false,
	wsUrl: "",
	token: "",
});

const saving = ref(false);
const reporterStatus = ref(false);
const panelRef = ref<HTMLElement | null>(null);

const loadSettings = () => {
	try {
		const saved = localStorage.getItem("owner-desktop-settings");
		if (saved) {
			settings.value = JSON.parse(saved);
		}
	} catch (error) {
		console.error("加载设置失败:", error);
	}
};

const checkReporterStatus = async () => {
	try {
		reporterStatus.value = await invoke<boolean>("get_reporter_status");
	} catch (error) {
		console.error("获取上报状态失败:", error);
	}
};

const saveSettings = async () => {
	saving.value = true;
	try {
		localStorage.setItem("owner-desktop-settings", JSON.stringify(settings.value));
		await invoke("update_reporter_config", {
			config: {
				enabled: settings.value.reportEnabled,
				ws_url: settings.value.wsUrl,
				token: settings.value.token,
			},
		});
		emit("save", settings.value);
		await checkReporterStatus();
		emit("close");
	} catch (error) {
		console.error("保存设置失败:", error);
	} finally {
		saving.value = false;
	}
};

loadSettings();
checkReporterStatus();

// Bottom Sheet 动画
const onPanelEnter = (el: Element, done: () => void) => {
	const panel = el as HTMLElement;
	gsap.set(panel, { y: "100%", opacity: 0 });
	gsap.to(panel, {
		y: "0%",
		opacity: 1,
		duration: 0.5,
		ease: "power3.out",
		onComplete: done,
	});
};

const onPanelLeave = (el: Element, done: () => void) => {
	gsap.to(el, {
		y: "100%",
		opacity: 0,
		duration: 0.35,
		ease: "power2.in",
		onComplete: done,
	});
};

watch(() => props.show, (show) => {
	if (show) {
		loadSettings();
		checkReporterStatus();
	}
});
</script>

<template>
	<Transition :css="false" @enter="onPanelEnter" @leave="onPanelLeave">
		<div v-if="show" ref="panelRef" class="settings-sheet" @click.self="emit('close')">
			<!-- 拖拽指示器 -->
			<div class="sheet-handle" @click="emit('close')">
				<span class="handle-bar" />
			</div>

			<!-- 标题区域 -->
			<div class="sheet-header">
				<h2 class="sheet-title">
					设置
				</h2>
				<button class="close-btn" @click="emit('close')">
					<Icon name="mdi:close" />
				</button>
			</div>

			<!-- 设置内容 -->
			<div class="sheet-content">
				<!-- 状态上报卡片 -->
				<div class="setting-card">
					<div class="card-header">
						<div class="card-icon">
							<Icon name="mdi:cloud-sync" />
						</div>
						<div class="card-info">
							<span class="card-title">状态上报</span>
							<span class="card-desc">向博客后端同步窗口和媒体信息</span>
						</div>
						<label class="toggle-switch">
							<input v-model="settings.reportEnabled" type="checkbox">
							<span class="slider" />
						</label>
					</div>

					<!-- 连接状态 -->
					<div v-if="settings.reportEnabled" class="connection-status">
						<span class="status-dot" :class="{ connected: reporterStatus }" />
						<span class="status-text">{{ reporterStatus ? "已连接" : "未连接" }}</span>
					</div>

					<!-- 展开的配置项 -->
					<Transition name="expand">
						<div v-if="settings.reportEnabled" class="card-expand">
							<div class="input-group">
								<label class="input-label">
									<Icon name="mdi:link-variant" />
									<span>服务器地址</span>
								</label>
								<input
									v-model="settings.wsUrl"
									type="text"
									class="text-input"
									placeholder="ws://example.com/api/ws/owner-desktop"
								>
							</div>

							<div class="input-group">
								<label class="input-label">
									<Icon name="mdi:key-outline" />
									<span>认证密钥</span>
								</label>
								<input
									v-model="settings.token"
									type="password"
									class="text-input"
									placeholder="输入你的 Token"
								>
							</div>
						</div>
					</Transition>
				</div>

				<!-- 提示信息 -->
				<div class="info-card">
					<Icon name="mdi:information-outline" class="info-icon" />
					<p>启用后将实时同步你的活动状态到博客，访客可以看到你正在做什么。</p>
				</div>
			</div>

			<!-- 底部操作区 -->
			<div class="sheet-footer">
				<button class="action-btn secondary" @click="emit('close')">
					取消
				</button>
				<button class="action-btn primary" :disabled="saving" @click="saveSettings">
					<Icon v-if="saving" name="mdi:loading" class="spin" />
					<span>{{ saving ? "保存中" : "保存" }}</span>
				</button>
			</div>
		</div>
	</Transition>
</template>

<style lang="scss" scoped>
$spring: cubic-bezier(0.34, 1.56, 0.64, 1);
$smooth: cubic-bezier(0.25, 0.8, 0.25, 1);

.settings-sheet {
	position: fixed;
	bottom: 0;
	left: 0;
	right: 0;
	max-height: 85vh;
	background: var(--bg-glass);
	backdrop-filter: blur(var(--blur-amount, 32px)) saturate(var(--blur-saturate, 180%));
	-webkit-backdrop-filter: blur(var(--blur-amount, 32px)) saturate(var(--blur-saturate, 180%));
	border: 1px solid var(--border-glass);
	border-bottom: none;
	border-radius: 24px 24px 0 0;
	box-shadow: 0 -8px 40px rgba(0, 0, 0, 0.15);
	z-index: 2000;
	display: flex;
	flex-direction: column;
	overflow: hidden;
}

.sheet-handle {
	display: flex;
	justify-content: center;
	padding: 12px 0 8px;
	cursor: pointer;

	.handle-bar {
		width: 36px;
		height: 4px;
		background: var(--text-tertiary);
		border-radius: 100px;
		opacity: 0.5;
		transition: all 0.2s ease;
	}

	&:hover .handle-bar {
		width: 48px;
		opacity: 0.8;
	}
}

.sheet-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	padding: 8px 20px 16px;
}

.sheet-title {
	margin: 0;
	font-size: 22px;
	font-weight: 700;
	color: var(--text-primary);
}

.close-btn {
	width: 32px;
	height: 32px;
	display: flex;
	align-items: center;
	justify-content: center;
	background: rgba(128, 128, 128, 0.1);
	border: none;
	border-radius: 50%;
	color: var(--text-secondary);
	cursor: pointer;
	font-size: 18px;
	transition: all 0.25s $spring;

	&:hover {
		background: rgba(255, 59, 48, 0.15);
		color: #ff3b30;
		transform: scale(1.1);
	}

	&:active {
		transform: scale(0.9);
	}
}

.sheet-content {
	flex: 1;
	padding: 0 20px;
	overflow-y: auto;
	display: flex;
	flex-direction: column;
	gap: 16px;
}

// 设置卡片
.setting-card {
	background: var(--bg-glass-hover);
	border: 1px solid var(--border-glass);
	border-radius: 20px;
	padding: 16px;
	transition: all 0.3s $smooth;
}

.card-header {
	display: flex;
	align-items: center;
	gap: 14px;
}

.card-icon {
	width: 44px;
	height: 44px;
	display: flex;
	align-items: center;
	justify-content: center;
	background: rgba(0, 122, 255, 0.12);
	border-radius: 12px;
	color: var(--accent-color);
	font-size: 22px;
}

.card-info {
	flex: 1;
	display: flex;
	flex-direction: column;
	gap: 2px;
}

.card-title {
	font-size: 16px;
	font-weight: 600;
	color: var(--text-primary);
}

.card-desc {
	font-size: 12px;
	color: var(--text-secondary);
}

// Toggle Switch
.toggle-switch {
	position: relative;
	width: 48px;
	height: 28px;
	cursor: pointer;

	input {
		opacity: 0;
		width: 0;
		height: 0;

		&:checked + .slider {
			background: var(--accent-color);

			&::before {
				transform: translateX(20px);
			}
		}
	}

	.slider {
		position: absolute;
		inset: 0;
		background: rgba(128, 128, 128, 0.3);
		border-radius: 100px;
		transition: all 0.3s $spring;

		&::before {
			content: "";
			position: absolute;
			height: 22px;
			width: 22px;
			left: 3px;
			bottom: 3px;
			background: white;
			border-radius: 50%;
			transition: transform 0.3s $spring;
			box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
		}
	}
}

// 连接状态
.connection-status {
	display: flex;
	align-items: center;
	gap: 8px;
	margin-top: 12px;
	padding-top: 12px;
	border-top: 1px solid var(--border-glass);
}

.status-dot {
	width: 8px;
	height: 8px;
	border-radius: 50%;
	background: #ff9500;

	&.connected {
		background: #34c759;
		box-shadow: 0 0 8px rgba(52, 199, 89, 0.5);
	}
}

.status-text {
	font-size: 13px;
	color: var(--text-secondary);
}

// 展开区域
.card-expand {
	margin-top: 16px;
	padding-top: 16px;
	border-top: 1px solid var(--border-glass);
	display: flex;
	flex-direction: column;
	gap: 14px;
}

.input-group {
	display: flex;
	flex-direction: column;
	gap: 8px;
}

.input-label {
	display: flex;
	align-items: center;
	gap: 6px;
	font-size: 13px;
	font-weight: 500;
	color: var(--text-secondary);

	.iconify {
		font-size: 16px;
	}
}

.text-input {
	width: 100%;
	padding: 12px 14px;
	background: rgba(128, 128, 128, 0.08);
	border: 1px solid var(--border-glass);
	border-radius: 12px;
	color: var(--text-primary);
	font-size: 14px;
	font-family: inherit;
	transition: all 0.25s ease;

	&:focus {
		outline: none;
		background: rgba(128, 128, 128, 0.12);
		border-color: var(--accent-color);
		box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
	}

	&::placeholder {
		color: var(--text-tertiary);
	}
}

// 展开动画
.expand-enter-active,
.expand-leave-active {
	transition: all 0.35s $smooth;
	overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
	opacity: 0;
	max-height: 0;
	margin-top: 0;
	padding-top: 0;
}

.expand-enter-to,
.expand-leave-from {
	opacity: 1;
	max-height: 200px;
}

// 提示卡片
.info-card {
	display: flex;
	gap: 12px;
	padding: 14px 16px;
	background: rgba(0, 122, 255, 0.08);
	border: 1px solid rgba(0, 122, 255, 0.15);
	border-radius: 16px;

	.info-icon {
		flex-shrink: 0;
		font-size: 18px;
		color: var(--accent-color);
		margin-top: 1px;
	}

	p {
		margin: 0;
		font-size: 13px;
		line-height: 1.5;
		color: var(--text-secondary);
	}
}

// 底部操作区
.sheet-footer {
	display: flex;
	gap: 12px;
	padding: 16px 20px 24px;
	border-top: 1px solid var(--border-glass);
	margin-top: 16px;
}

.action-btn {
	flex: 1;
	display: flex;
	align-items: center;
	justify-content: center;
	gap: 8px;
	padding: 14px 20px;
	border: none;
	border-radius: 14px;
	font-size: 15px;
	font-weight: 600;
	cursor: pointer;
	transition: all 0.3s $spring;

	&:active {
		transform: scale(0.96);
	}

	&.secondary {
		background: rgba(128, 128, 128, 0.12);
		color: var(--text-primary);

		&:hover {
			background: rgba(128, 128, 128, 0.2);
		}
	}

	&.primary {
		background: var(--accent-color);
		color: white;
		box-shadow: 0 4px 12px rgba(0, 122, 255, 0.25);

		&:hover:not(:disabled) {
			box-shadow: 0 6px 20px rgba(0, 122, 255, 0.35);
			transform: translateY(-1px);
		}

		&:disabled {
			opacity: 0.6;
			cursor: not-allowed;
		}
	}
}

.spin {
	animation: spin 1s linear infinite;
}

@keyframes spin {
	to {
		transform: rotate(360deg);
	}
}
</style>
