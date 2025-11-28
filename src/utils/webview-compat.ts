/**
 * WebView 兼容性检测与修复工具
 * 用于检测和处理 Tauri WebView 中的 backdrop-filter 问题
 */

/**
 * 检测 backdrop-filter 是否被支持且正常工作
 */
export function checkBackdropFilterSupport(): boolean {
	// 基础支持检测
	const testEl = document.createElement("div");
	testEl.style.cssText = "-webkit-backdrop-filter: blur(1px); backdrop-filter: blur(1px);";

	// 使用类型断言访问 webkit 前缀属性
	const style = testEl.style as CSSStyleDeclaration & { webkitBackdropFilter?: string };
	const hasBasicSupport
		= testEl.style.backdropFilter !== ""
			|| (style.webkitBackdropFilter !== undefined && style.webkitBackdropFilter !== "");

	return hasBasicSupport;
}

/**
 * 强制触发 GPU 层重绘
 * 用于修复构建后 backdrop-filter 消失的问题
 */
export function forceGPURepaint(): void {
	// 获取所有使用了 backdrop-filter 的元素
	const glassElements = document.querySelectorAll(
		".foreground-widget, .media-widget, .capsule-menu, .glass-effect",
	);

	glassElements.forEach((el) => {
		const htmlEl = el as HTMLElement;
		// 强制触发重绘
		htmlEl.style.transform = "translate3d(0, 0, 0.1px)";

		// 使用 requestAnimationFrame 确保变化被应用
		requestAnimationFrame(() => {
			htmlEl.style.transform = "";
		});
	});
}

/**
 * 应用降级样式
 * 当检测到 backdrop-filter 不工作时使用
 */
export function applyFallbackStyles(): void {
	document.documentElement.classList.add("backdrop-filter-fallback");

	// 添加降级样式
	const style = document.createElement("style");
	style.id = "backdrop-filter-fallback-styles";
	style.textContent = `
    .backdrop-filter-fallback .foreground-widget,
    .backdrop-filter-fallback .media-widget,
    .backdrop-filter-fallback .capsule-menu,
    .backdrop-filter-fallback .glass-effect {
      background: var(--bg-glass-fallback) !important;
      -webkit-backdrop-filter: none !important;
      backdrop-filter: none !important;
    }
  `;

	// 避免重复添加
	if (!document.getElementById("backdrop-filter-fallback-styles")) {
		document.head.appendChild(style);
	}
}

/**
 * 移除降级样式
 */
export function removeFallbackStyles(): void {
	document.documentElement.classList.remove("backdrop-filter-fallback");

	const fallbackStyle = document.getElementById("backdrop-filter-fallback-styles");
	if (fallbackStyle) {
		fallbackStyle.remove();
	}
}

/**
 * 初始化 WebView 兼容性修复
 * 在应用启动时调用
 */
export function initWebViewFixes(): void {
	// 检测支持情况
	const isSupported = checkBackdropFilterSupport();

	if (!isSupported) {
		console.warn("[WebView] backdrop-filter 不被支持，应用降级样式");
		applyFallbackStyles();
		return;
	}

	// 即使支持，也强制触发一次重绘以确保效果生效
	// 使用延迟确保 DOM 已完全渲染
	setTimeout(() => {
		forceGPURepaint();
	}, 100);

	// 监听窗口可见性变化，在窗口重新可见时强制重绘
	document.addEventListener("visibilitychange", () => {
		if (document.visibilityState === "visible") {
			forceGPURepaint();
		}
	});

	// 监听窗口 focus 事件，有时 WebView 在失焦后会丢失 backdrop-filter
	window.addEventListener("focus", () => {
		setTimeout(forceGPURepaint, 50);
	});
}

/**
 * 创建带有正确 GPU 加速属性的玻璃效果样式对象
 * 可用于内联样式
 */
export function createGlassStyle(blurAmount: number = 24): Record<string, string> {
	return {
		"isolation": "isolate",
		"-webkit-transform": "translate3d(0, 0, 0)",
		"transform": "translate3d(0, 0, 0)",
		"-webkit-backface-visibility": "hidden",
		"backface-visibility": "hidden",
		"will-change": "backdrop-filter, -webkit-backdrop-filter, transform",
		"-webkit-backdrop-filter": `blur(${blurAmount}px) saturate(180%)`,
		"backdrop-filter": `blur(${blurAmount}px) saturate(180%)`,
	};
}

export default {
	checkBackdropFilterSupport,
	forceGPURepaint,
	applyFallbackStyles,
	removeFallbackStyles,
	initWebViewFixes,
	createGlassStyle,
};
