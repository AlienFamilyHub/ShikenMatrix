<script setup lang="ts">
import gsap from "gsap";
import { computed, ref, watch } from "vue";

interface WindowInfo {
	title: string;
	process_name: string;
	icon_data?: number[];
	app_id?: string;
}

const props = defineProps<{
	info: WindowInfo | null;
}>();

const iconUrl = ref<string | null>(null);
const prevBundleId = ref<string | null>(null);

// 监听 info 变化，只在应用切换时更新图标
watch(
	() => props.info,
	(newInfo) => {
		if (!newInfo) {
			iconUrl.value = null;
			prevBundleId.value = null;
			return;
		}

		// 只有当 bundle_id 变化时才更新图标
		const newBundleId = newInfo.app_id || newInfo.process_name;
		if (newBundleId !== prevBundleId.value) {
			prevBundleId.value = newBundleId;
			if (newInfo.icon_data && newInfo.icon_data.length > 0) {
				// 释放旧的 blob URL
				if (iconUrl.value) {
					URL.revokeObjectURL(iconUrl.value);
				}
				const blob = new Blob([new Uint8Array(newInfo.icon_data)], { type: "image/png" });
				iconUrl.value = URL.createObjectURL(blob);
			} else {
				iconUrl.value = null;
			}
		}
	},
	{ immediate: true },
);

// 直接显示完整标题，不截断
const displayTitle = computed(() => props.info?.title || "");

// GSAP 动画钩子 - 图标
const onIconEnter = (el: Element, done: () => void) => {
	gsap.fromTo(el, { opacity: 0 }, { opacity: 1, duration: 0.2, ease: "power2.out", onComplete: done });
};
const onIconLeave = (el: Element, done: () => void) => {
	gsap.to(el, { opacity: 0, duration: 0.15, ease: "power2.in", onComplete: done });
};

// GSAP 动画钩子 - 文字
const onTextEnter = (el: Element, done: () => void) => {
	gsap.fromTo(el, { opacity: 0 }, { opacity: 1, duration: 0.15, ease: "power1.out", onComplete: done });
};
const onTextLeave = (el: Element, done: () => void) => {
	gsap.to(el, { opacity: 0, duration: 0.1, ease: "power1.in", onComplete: done });
};
</script>

<template>
	<div class="foreground-widget">
		<div class="widget-content">
			<!-- 图标区域 -->
			<div class="icon-wrapper">
				<div class="icon-glow" />
				<div class="icon-container">
					<Transition mode="out-in" :css="false" @enter="onIconEnter" @leave="onIconLeave">
						<img v-if="iconUrl" :key="iconUrl" :src="iconUrl" alt="App Icon" class="app-icon">
						<div v-else class="app-icon-placeholder">
							<Icon name="mdi:application" class="placeholder-icon" />
						</div>
					</Transition>
				</div>
			</div>

			<!-- 信息区域 -->
			<div class="info-section">
				<Transition mode="out-in" :css="false" @enter="onTextEnter" @leave="onTextLeave">
					<h2 v-if="info" :key="info.process_name" class="app-name">
						{{ info.process_name }}
					</h2>
					<h2 v-else class="app-name placeholder">
						等待中...
					</h2>
				</Transition>

				<Transition mode="out-in" :css="false" @enter="onTextEnter" @leave="onTextLeave">
					<p v-if="info?.title" :key="info.title" class="window-title">
						{{ displayTitle }}
					</p>
					<p v-else class="window-title placeholder">
						暂无窗口信息
					</p>
				</Transition>
			</div>
		</div>

		<!-- 装饰元素 -->
		<div class="decoration-line" />
	</div>
</template>

<style lang="scss" scoped>
$transition-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
$max-width: 450px;

.foreground-widget {
	position: relative;
	width: 100%;
	padding: 28px;
	box-sizing: border-box;
	background: var(--bg-glass);
	border: 1px solid var(--border-glass);
	border-radius: 28px;

	/* GPU 加速 - 防止 backdrop-filter 失效 */
	isolation: isolate;
	-webkit-transform: translate3d(0, 0, 0);
	transform: translate3d(0, 0, 0);
	-webkit-backface-visibility: hidden;
	backface-visibility: hidden;
	will-change:
		backdrop-filter,
		-webkit-backdrop-filter,
		transform;

	/* 双写 backdrop-filter 确保兼容性 */
	-webkit-backdrop-filter: blur(var(--blur-amount, 24px)) saturate(var(--blur-saturate, 180%));
	backdrop-filter: blur(var(--blur-amount, 24px)) saturate(var(--blur-saturate, 180%));

	overflow: hidden;
	transition: all 0.4s $transition-spring;

	&:hover {
		transform: translate3d(0, -4px, 0) scale(1.01);
		background: var(--bg-glass-hover);

		.icon-glow {
			opacity: 0.25;
		}

		.decoration-line {
			opacity: 0.6;
			width: 60px;
		}
	}
}

/* 降级方案 - 当 backdrop-filter 不支持时 */
@supports not ((-webkit-backdrop-filter: blur(1px)) or (backdrop-filter: blur(1px))) {
	.foreground-widget {
		background: var(--bg-glass-fallback);
	}
}

.widget-content {
	display: flex;
	flex-direction: row;
	align-items: center;
	gap: 20px;
}

// 图标样式
.icon-wrapper {
	position: relative;
	display: flex;
	align-items: center;
	justify-content: center;
	flex-shrink: 0;
}

.icon-glow {
	position: absolute;
	width: 100px;
	height: 100px;
	background: radial-gradient(circle, var(--accent-color) 0%, transparent 70%);
	opacity: 0.15;
	filter: blur(20px);
	transition: opacity 0.4s ease;
}

.icon-container {
	width: 72px;
	height: 72px;
	display: flex;
	align-items: center;
	justify-content: center;
	background: rgba(255, 255, 255, 0.1);
	border-radius: 18px;
	overflow: hidden;
	box-shadow:
		0 4px 12px rgba(0, 0, 0, 0.08),
		inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.app-icon {
	width: 56px;
	height: 56px;
	object-fit: contain;
	filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.15));
	/* 抗锯齿优化 */
	image-rendering: -webkit-optimize-contrast;
	image-rendering: high-quality;
	transform: translateZ(0);
}

.app-icon-placeholder {
	width: 100%;
	height: 100%;
	display: flex;
	align-items: center;
	justify-content: center;
	background: linear-gradient(135deg, rgba(120, 120, 128, 0.2), rgba(120, 120, 128, 0.1));

	.placeholder-icon {
		font-size: 32px;
		color: var(--text-tertiary);
	}
}

// 信息区域
.info-section {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	gap: 6px;
	width: 100%;
	text-align: left;
	min-width: 0; /* 确保文本截断生效 */
}

.app-name {
	margin: 0;
	font-size: 20px;
	font-weight: 700;
	color: var(--text-primary);
	letter-spacing: -0.3px;
	line-height: 1.3;
	white-space: nowrap;
	text-align: left;
	width: 100%;
	overflow: hidden;
	text-overflow: ellipsis;

	&.placeholder {
		color: var(--text-tertiary);
		font-weight: 500;
	}
}

.window-title {
	margin: 0;
	font-size: 13px;
	font-weight: 400;
	color: var(--text-secondary);
	line-height: 1.5;
	white-space: nowrap;
	text-align: left;
	width: 100%;
	overflow: hidden;
	text-overflow: ellipsis;

	&.placeholder {
		color: var(--text-tertiary);
	}
}

// 超过最大宽度时允许换行
@media (max-width: $max-width) {
	.app-name,
	.window-title {
		white-space: normal;
		word-break: break-word;
	}
}

// 装饰线
.decoration-line {
	position: absolute;
	bottom: 0;
	left: 50%;
	transform: translateX(-50%);
	width: 40px;
	height: 4px;
	background: linear-gradient(90deg, transparent, var(--accent-color), transparent);
	border-radius: 2px;
	opacity: 0;
	transition:
		opacity 0.3s ease,
		width 0.3s ease;
}
</style>
