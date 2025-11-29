<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import gsap from "gsap";
import { nextTick, onMounted, onUnmounted, ref } from "vue";
import "@/assets/main.css";

const isDragging = ref(false);
const showCapsuleMenu = ref(false);
const isPinned = ref(false);
const capsuleRef = ref<HTMLElement | null>(null);

// 长按相关状态
const longPressTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const isLongPressing = ref(false);
const longPressTriggered = ref(false); // 标记长按是否已触发
const LONG_PRESS_DURATION = 500; // 长按触发时间（毫秒）

// 使用 Tauri API 实现窗口拖动
const startDrag = async () => {
	isDragging.value = true;
	try {
		const window = getCurrentWindow();
		await window.startDragging();
	} catch (e) {
		console.error("拖动失败:", e);
	} finally {
		// 延迟重置状态，让动画有时间播放
		setTimeout(() => {
			isDragging.value = false;
		}, 150);
	}
};

// 打开胶囊菜单（带动画）
const openCapsuleMenu = async () => {
	if (showCapsuleMenu.value)
		return;

	showCapsuleMenu.value = true;
	await nextTick();
	if (capsuleRef.value) {
		gsap.set(capsuleRef.value, { transformOrigin: "center top" });
		gsap.fromTo(capsuleRef.value, {
			opacity: 0,
			scale: 0.3,
			y: -30,
			rotateX: 45,
			filter: "blur(8px)",
		}, {
			opacity: 1,
			scale: 1,
			y: 0,
			rotateX: 0,
			filter: "blur(0px)",
			duration: 0.45,
			ease: "back.out(1.4)",
		});
		const buttons = capsuleRef.value.querySelectorAll(".capsule-btn");
		gsap.fromTo(buttons, {
			opacity: 0,
			scale: 0,
			y: -10,
		}, {
			opacity: 1,
			scale: 1,
			y: 0,
			duration: 0.3,
			stagger: 0.04,
			ease: "back.out(2.5)",
			delay: 0.1,
		});
	}
};

// 关闭胶囊菜单（带动画）
const closeCapsuleMenu = () => {
	if (capsuleRef.value) {
		// 按钮先收起
		const buttons = capsuleRef.value.querySelectorAll(".capsule-btn");
		gsap.to(buttons, {
			opacity: 0,
			scale: 0,
			y: -10,
			duration: 0.15,
			stagger: 0.02,
			ease: "power2.in",
		});

		// 整个胶囊收起
		gsap.to(capsuleRef.value, {
			opacity: 0,
			scale: 0.3,
			y: -30,
			rotateX: 45,
			filter: "blur(8px)",
			duration: 0.25,
			delay: 0.05,
			ease: "back.in(1.7)",
			onComplete: () => {
				showCapsuleMenu.value = false;
			},
		});
	} else {
		showCapsuleMenu.value = false;
	}
};

// 清除长按计时器
const clearLongPressTimer = () => {
	if (longPressTimer.value) {
		clearTimeout(longPressTimer.value);
		longPressTimer.value = null;
	}
	isLongPressing.value = false;
};

// 重置长按状态（不清除计时器）
const resetLongPressState = () => {
	isLongPressing.value = false;
	longPressTriggered.value = false;
};

// 双击打开/关闭胶囊菜单
const handleDoubleClick = async () => {
	// 清除长按计时器
	clearLongPressTimer();

	if (showCapsuleMenu.value) {
		closeCapsuleMenu();
	} else {
		await openCapsuleMenu();
	}
};

// 长按开始 - 同时启动拖动和长按计时
const handleLongPressStart = (_e: MouseEvent | TouchEvent) => {
	// 清除之前的计时器
	clearLongPressTimer();
	longPressTriggered.value = false;

	// 立即启动拖动
	startDrag();

	isLongPressing.value = true;
	longPressTimer.value = setTimeout(async () => {
		// 长按触发，切换菜单状态
		longPressTriggered.value = true;
		if (showCapsuleMenu.value) {
			closeCapsuleMenu();
		} else {
			await openCapsuleMenu();
		}
		isLongPressing.value = false;
	}, LONG_PRESS_DURATION);
};

// 长按结束
const handleLongPressEnd = () => {
	// 如果长按已经触发（菜单已打开/关闭），只重置视觉状态
	if (longPressTriggered.value) {
		resetLongPressState();
		return;
	}
	// 如果长按未触发，清除计时器
	clearLongPressTimer();
};

// 窗口失焦时关闭菜单
const handleWindowBlur = () => {
	if (showCapsuleMenu.value) {
		closeCapsuleMenu();
	}
};

onMounted(() => {
	window.addEventListener("blur", handleWindowBlur);
});

// 组件卸载时清除计时器和事件监听
onUnmounted(() => {
	clearLongPressTimer();
	window.removeEventListener("blur", handleWindowBlur);
});

// 窗口操作
const handleClose = async () => {
	closeCapsuleMenu();
	setTimeout(async () => {
		const window = getCurrentWindow();
		await window.close();
	}, 200);
};

const handleMinimize = async () => {
	closeCapsuleMenu();
	const window = getCurrentWindow();
	await window.minimize();
};

const handlePin = async () => {
	const window = getCurrentWindow();
	isPinned.value = !isPinned.value;
	await window.setAlwaysOnTop(isPinned.value);
};

// 点击外部关闭菜单
const handleClickOutside = (e: MouseEvent) => {
	if (!showCapsuleMenu.value)
		return;
	const target = e.target as HTMLElement;
	if (!target.closest(".capsule-menu")) {
		closeCapsuleMenu();
	}
};
</script>

<template>
	<div class="app-container" @click="handleClickOutside">
		<!-- 顶部拖拽区域 - 双击或长按打开菜单 -->
		<div
			class="drag-handle" :class="{ 'dragging': isDragging, 'long-pressing': isLongPressing }"
			@mousedown="handleLongPressStart" @mouseup="handleLongPressEnd" @mouseleave="handleLongPressEnd"
			@touchstart.passive="handleLongPressStart" @touchend="handleLongPressEnd" @touchcancel="handleLongPressEnd"
			@dblclick.stop="handleDoubleClick"
			@click.stop
		>
			<div class="drag-indicator">
				<span class="indicator-dot" />
				<span class="indicator-line" />
				<span class="indicator-dot" />
			</div>
		</div>

		<!-- 胶囊操作菜单 -->
		<div v-if="showCapsuleMenu" ref="capsuleRef" class="capsule-menu" @click.stop>
			<button class="capsule-btn close" title="关闭" @click="handleClose">
				<Icon name="mdi:close" />
			</button>
			<button class="capsule-btn minimize" title="最小化" @click="handleMinimize">
				<Icon name="mdi:minus" />
			</button>
			<button class="capsule-btn pin" :class="{ active: isPinned }" title="置顶" @click="handlePin">
				<Icon name="mdi:pin" />
			</button>
		</div>

		<div class="content-area">
			<NuxtPage />
		</div>
	</div>
</template>

<style lang="scss">
$transition-spring: cubic-bezier(0.34, 1.56, 0.64, 1);

.app-container {
	width: 100%;
	height: 100vh;
	display: flex;
	flex-direction: column;
	background: transparent;
	overflow: hidden;
	/* 创建独立堆叠上下文 */
	isolation: isolate;
}

.drag-handle {
	width: 100%;
	height: 36px;
	display: flex;
	align-items: center;
	justify-content: center;
	cursor: grab;
	flex-shrink: 0;
	position: relative;
	z-index: 1000;

	&:active,
	&.dragging {
		cursor: grabbing;
	}

	// 指示器容器
	.drag-indicator {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 12px;
		border-radius: 100px;
		transition: all 0.25s $transition-spring;
	}

	&:hover .drag-indicator {
		background: rgba(128, 128, 128, 0.1);
		transform: scale(1.05);
	}

	&:active .drag-indicator,
	&.dragging .drag-indicator {
		background: rgba(128, 128, 128, 0.15);
		transform: scale(0.95);
	}

	// 小圆点
	.indicator-dot {
		width: 5px;
		height: 5px;
		background: var(--text-tertiary);
		border-radius: 50%;
		opacity: 0;
		transform: scale(0);
		transition: all 0.25s $transition-spring;
	}

	&:hover .indicator-dot {
		opacity: 0.6;
		transform: scale(1);
	}

	&:active .indicator-dot,
	&.dragging .indicator-dot {
		opacity: 0.8;
		transform: scale(1.2);
		background: var(--accent-color);
	}

	// 横线
	.indicator-line {
		width: 36px;
		height: 5px;
		background: var(--text-tertiary);
		border-radius: 100px;
		opacity: 0.4;
		transition: all 0.25s $transition-spring;
	}

	&:hover .indicator-line {
		width: 44px;
		opacity: 0.6;
		background: var(--text-secondary);
	}

	&:active .indicator-line,
	&.dragging .indicator-line {
		width: 32px;
		opacity: 0.9;
		background: var(--accent-color);
		box-shadow: 0 0 8px rgba(0, 122, 255, 0.4);
	}

	// 长按状态 - 渐变发光效果
	&.long-pressing .drag-indicator {
		background: rgba(0, 122, 255, 0.15);
		transform: scale(1.08);
		animation: long-press-pulse 0.5s ease-in-out infinite;
	}

	&.long-pressing .indicator-dot {
		opacity: 1;
		transform: scale(1.3);
		background: var(--accent-color);
		animation: long-press-dot-pulse 0.5s ease-in-out infinite;
	}

	&.long-pressing .indicator-line {
		width: 48px;
		opacity: 1;
		background: linear-gradient(90deg, var(--accent-color), #5ac8fa, var(--accent-color));
		background-size: 200% 100%;
		animation: long-press-gradient 1s linear infinite;
		box-shadow: 0 0 12px rgba(0, 122, 255, 0.5);
	}
}

// 长按动画关键帧
@keyframes long-press-pulse {
	0%,
	100% {
		transform: scale(1.05);
		background: rgba(0, 122, 255, 0.12);
	}
	50% {
		transform: scale(1.1);
		background: rgba(0, 122, 255, 0.2);
	}
}

@keyframes long-press-dot-pulse {
	0%,
	100% {
		transform: scale(1.2);
	}
	50% {
		transform: scale(1.4);
	}
}

@keyframes long-press-gradient {
	0% {
		background-position: 0% 50%;
	}
	100% {
		background-position: 200% 50%;
	}
}

// 胶囊操作菜单
.capsule-menu {
	position: absolute;
	top: 42px; // 紧贴拖拽指示器下方
	left: 0;
	right: 0;
	margin: 0 auto;
	width: fit-content;
	display: flex;
	align-items: center;
	gap: 8px;
	padding: 8px 12px;
	background: var(--bg-glass-elevated);

	/* GPU 加速 - 防止 backdrop-filter 失效 */
	isolation: isolate;
	-webkit-backface-visibility: hidden;
	backface-visibility: hidden;
	will-change:
		backdrop-filter,
		-webkit-backdrop-filter,
		transform,
		opacity;

	/* 双写 backdrop-filter 确保兼容性 */
	-webkit-backdrop-filter: blur(var(--blur-amount, 24px)) saturate(var(--blur-saturate, 180%));
	backdrop-filter: blur(var(--blur-amount, 24px)) saturate(var(--blur-saturate, 180%));

	border: 1px solid var(--border-glass);
	border-radius: 100px;
	box-shadow:
		0 8px 32px rgba(0, 0, 0, 0.12),
		0 2px 8px rgba(0, 0, 0, 0.08);
	z-index: 1001;
}

/* 降级方案 */
@supports not ((-webkit-backdrop-filter: blur(1px)) or (backdrop-filter: blur(1px))) {
	.capsule-menu {
		background: var(--bg-glass-fallback-elevated);
	}
}

.capsule-btn {
	display: flex;
	align-items: center;
	justify-content: center;
	width: 32px;
	height: 32px;
	border: none;
	border-radius: 50%;
	background: rgba(128, 128, 128, 0.1);
	color: var(--text-secondary);
	cursor: pointer;
	font-size: 16px;
	will-change: transform, background-color;
	transform: translateZ(0); // 启用 GPU 加速
	backface-visibility: hidden;
	transition:
		transform 0.25s $transition-spring,
		background-color 0.2s ease-out,
		color 0.2s ease-out;

	&:hover {
		transform: translateZ(0) scale(1.1);
		background: rgba(128, 128, 128, 0.2);
		color: var(--text-primary);
	}

	&:active {
		transform: translateZ(0) scale(0.95);
		transition-duration: 0.1s;
	}

	// 关闭按钮
	&.close:hover {
		background: rgba(255, 59, 48, 0.2);
		color: #ff3b30;
	}

	// 最小化按钮
	&.minimize:hover {
		background: rgba(255, 204, 0, 0.2);
		color: #ffcc00;
	}

	// 置顶按钮
	&.pin:hover {
		background: rgba(0, 122, 255, 0.2);
		color: var(--accent-color);
	}

	&.pin.active {
		background: rgba(0, 122, 255, 0.25);
		color: var(--accent-color);

		&:hover {
			background: rgba(0, 122, 255, 0.35);
		}
	}
}

.content-area {
	flex: 1;
	overflow: auto;
	/* 创建独立堆叠上下文，确保子元素 backdrop-filter 正常工作 */
	isolation: isolate;
	-webkit-transform: translateZ(0);
	transform: translateZ(0);
}
</style>
