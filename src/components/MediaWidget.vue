<script setup lang="ts">
import gsap from "gsap";
import { computed } from "vue";

interface MediaMetadata {
	title?: string;
	artist?: string;
	album?: string;
	artwork_data?: string; // Base64 string
	artwork_mime_type?: string;
}

interface PlaybackState {
	playing: boolean;
}

const props = defineProps<{
	metadata: MediaMetadata | null;
	playbackState: PlaybackState | null;
}>();

const isVisible = computed(() => {
	if (!props.metadata)
		return false;
	const hasContent = !!(props.metadata.title || props.metadata.artist);
	const isPlaying = props.playbackState?.playing ?? false;
	return hasContent && isPlaying;
});

const artworkUrl = computed(() => {
	if (props.metadata?.artwork_data && props.metadata?.artwork_mime_type) {
		return `data:${props.metadata.artwork_mime_type};base64,${props.metadata.artwork_data}`;
	}
	return null;
});

// GSAP 动画钩子 - 组件入场/出场
const onWidgetEnter = (el: Element, done: () => void) => {
	gsap.fromTo(el, { opacity: 0, y: 12, scale: 0.98 }, { opacity: 1, y: 0, scale: 1, duration: 0.35, ease: "power2.out", onComplete: done });
};
const onWidgetLeave = (el: Element, done: () => void) => {
	gsap.to(el, { opacity: 0, y: -8, scale: 0.98, duration: 0.25, ease: "power2.in", onComplete: done });
};

// GSAP 动画钩子 - 封面
const onArtworkEnter = (el: Element, done: () => void) => {
	gsap.fromTo(el, { opacity: 0 }, { opacity: 1, duration: 0.2, ease: "power2.out", onComplete: done });
};
const onArtworkLeave = (el: Element, done: () => void) => {
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
	<Transition :css="false" @enter="onWidgetEnter" @leave="onWidgetLeave">
		<div v-if="isVisible && metadata" class="media-widget">
			<!-- 背景模糊层 -->
			<div v-if="artworkUrl" class="artwork-blur-bg" :style="{ backgroundImage: `url(${artworkUrl})` }" />

			<div class="widget-content">
				<!-- 封面区域 -->
				<div class="artwork-wrapper">
					<div class="artwork-glow" />
					<div class="artwork-container">
						<Transition mode="out-in" :css="false" @enter="onArtworkEnter" @leave="onArtworkLeave">
							<img
								v-if="artworkUrl"
								:key="artworkUrl"
								:src="artworkUrl"
								alt="Album Artwork"
								class="artwork"
							>
							<div v-else class="artwork-placeholder">
								<Icon name="mdi:music-note" class="music-icon" />
							</div>
						</Transition>
					</div>
					<!-- 播放动画指示器 -->
					<div class="playing-indicator">
						<span class="bar" />
						<span class="bar" />
						<span class="bar" />
					</div>
				</div>

				<!-- 信息区域 -->
				<div class="info-section">
					<Transition mode="out-in" :css="false" @enter="onTextEnter" @leave="onTextLeave">
						<h3 :key="metadata.title" class="track-title">
							{{ metadata.title || "未知标题" }}
						</h3>
					</Transition>

					<Transition mode="out-in" :css="false" @enter="onTextEnter" @leave="onTextLeave">
						<p :key="metadata.artist" class="artist-name">
							{{ metadata.artist || "未知艺术家" }}
						</p>
					</Transition>

					<Transition mode="out-in" :css="false" @enter="onTextEnter" @leave="onTextLeave">
						<p v-if="metadata.album" :key="metadata.album" class="album-name">
							{{ metadata.album }}
						</p>
					</Transition>
				</div>
			</div>

			<!-- 装饰元素 -->
			<div class="decoration-dots">
				<span /><span /><span />
			</div>
		</div>
	</Transition>
</template>

<style lang="scss" scoped>
$transition-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
$max-width: 450px;

.media-widget {
	position: relative;
	width: 100%;
	padding: 24px;
	box-sizing: border-box;
	background: var(--bg-glass);
	border: 1px solid var(--border-glass);
	border-radius: 28px;
	/* box-shadow: var(--shadow-card); */

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
		/* box-shadow: 0 12px 40px rgba(0, 0, 0, 0.12); */

		.artwork-glow {
			opacity: 0.35;
		}

		.artwork-container {
			transform: scale(1.05);
		}

		.decoration-dots {
			opacity: 0.6;
		}
	}
}

/* 降级方案 - 当 backdrop-filter 不支持时 */
@supports not ((-webkit-backdrop-filter: blur(1px)) or (backdrop-filter: blur(1px))) {
	.media-widget {
		background: var(--bg-glass-fallback);
	}
}

// 背景模糊封面
.artwork-blur-bg {
	position: absolute;
	top: -50%;
	left: -50%;
	right: -50%;
	bottom: -50%;
	background-size: cover;
	background-position: center;
	/* GPU 加速的 filter */
	-webkit-filter: blur(60px) saturate(1.5);
	filter: blur(60px) saturate(1.5);
	opacity: 0.3;
	/* 使用 translate3d 替代 scale 以保持 GPU 层 */
	-webkit-transform: translate3d(0, 0, 0) scale(2);
	transform: translate3d(0, 0, 0) scale(2);
	pointer-events: none;
	/* 隔离层防止影响父元素 */
	isolation: isolate;
}

.widget-content {
	position: relative;
	display: flex;
	flex-direction: row;
	align-items: center;
	gap: 20px;
	z-index: 1;
}

// 封面样式
.artwork-wrapper {
	position: relative;
	flex-shrink: 0;
}

.artwork-glow {
	position: absolute;
	top: 50%;
	left: 50%;
	transform: translate(-50%, -50%);
	width: 100px;
	height: 100px;
	background: radial-gradient(circle, var(--accent-color) 0%, transparent 70%);
	opacity: 0.2;
	filter: blur(20px);
	transition: opacity 0.4s ease;
}

.artwork-container {
	width: 72px;
	height: 72px;
	border-radius: 16px;
	overflow: hidden;
	box-shadow:
		0 8px 24px rgba(0, 0, 0, 0.15),
		inset 0 1px 0 rgba(255, 255, 255, 0.1);
	transition: transform 0.3s $transition-spring;
}

.artwork {
	width: 100%;
	height: 100%;
	object-fit: cover;
	/* 抗锯齿优化 */
	image-rendering: -webkit-optimize-contrast;
	image-rendering: high-quality;
	transform: translateZ(0);
}

.artwork-placeholder {
	width: 100%;
	height: 100%;
	background: linear-gradient(135deg, rgba(120, 120, 128, 0.3), rgba(120, 120, 128, 0.1));
	display: flex;
	align-items: center;
	justify-content: center;

	.music-icon {
		font-size: 28px;
		color: var(--text-tertiary);
	}
}

// 播放指示器
.playing-indicator {
	position: absolute;
	bottom: -8px;
	left: 50%;
	transform: translateX(-50%);
	display: flex;
	gap: 3px;
	align-items: flex-end;
	height: 16px;

	.bar {
		width: 4px;
		background: var(--accent-color);
		border-radius: 2px;
		animation: equalizer 0.8s ease-in-out infinite;

		&:nth-child(1) {
			height: 8px;
			animation-delay: 0s;
		}

		&:nth-child(2) {
			height: 14px;
			animation-delay: 0.2s;
		}

		&:nth-child(3) {
			height: 10px;
			animation-delay: 0.4s;
		}
	}
}

@keyframes equalizer {
	0%,
	100% {
		transform: scaleY(0.5);
	}
	50% {
		transform: scaleY(1);
	}
}

// 信息区域
.info-section {
	display: flex;
	flex-direction: column;
	gap: 4px;
	flex: 1;
	min-width: 0;
}

.track-title {
	margin: 0;
	font-size: 16px;
	font-weight: 700;
	color: var(--text-primary);
	letter-spacing: -0.3px;
	line-height: 1.3;
	white-space: nowrap;
}

.artist-name {
	margin: 0;
	font-size: 13px;
	font-weight: 500;
	color: var(--text-secondary);
	line-height: 1.4;
	white-space: nowrap;
}

.album-name {
	margin: 0;
	font-size: 12px;
	font-weight: 400;
	color: var(--text-tertiary);
	line-height: 1.4;
	white-space: nowrap;
}

// 超过最大宽度时允许换行
@media (max-width: $max-width) {
	.track-title,
	.artist-name,
	.album-name {
		white-space: normal;
		word-break: break-word;
	}
}

// 装饰圆点
.decoration-dots {
	position: absolute;
	top: 16px;
	right: 16px;
	display: flex;
	gap: 4px;
	opacity: 0.4;
	transition: opacity 0.3s ease;

	span {
		width: 6px;
		height: 6px;
		background: var(--text-tertiary);
		border-radius: 50%;
	}
}
</style>
