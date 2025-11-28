/**
 * WebView 兼容性修复插件
 * 自动初始化 backdrop-filter 修复
 */
import { initWebViewFixes } from "@/utils/webview-compat";

export default defineNuxtPlugin(() => {
	// 仅在客户端运行
	if (import.meta.client) {
		// 在 DOM 就绪后初始化
		if (document.readyState === "complete") {
			initWebViewFixes();
		} else {
			window.addEventListener("load", () => {
				initWebViewFixes();
			});
		}
	}
});
