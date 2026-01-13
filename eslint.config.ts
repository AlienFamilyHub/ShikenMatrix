import antfu from "@antfu/eslint-config";
import { defineConfigWithVueTs, vueTsConfigs } from "@vue/eslint-config-typescript";
import pluginOxlint from "eslint-plugin-oxlint";
import pluginVue from "eslint-plugin-vue";
import { globalIgnores } from "eslint/config";

const atfConfig = antfu({
	formatters: true,
	unocss: true,
	vue: true,
	stylistic: {
		indent: "tab",
		quotes: "double",
		semi: true,
	},
	rules: {
		// 忽略 antfu/top-level-function 规则
		"antfu/top-level-function": "off",
		// no-console 允许info和warn、error
		"no-console": ["error", { allow: ["info", "warn", "error"] }],
		"brace-style": ["error", "1tbs", { allowSingleLine: true }],
	},
});

// To allow more languages other than `ts` in `.vue` files, uncomment the following lines:
// import { configureVueProject } from '@vue/eslint-config-typescript'
// configureVueProject({ scriptLangs: ['ts', 'tsx'] })
// More info at https://github.com/vuejs/eslint-config-typescript/#advanced-setup

export default defineConfigWithVueTs(
	{
		name: "app/files-to-lint",
		files: ["**/*.{ts,mts,tsx,vue}"],
	},

	globalIgnores(["**/dist/**", "**/dist-ssr/**", "**/coverage/**"]),

	pluginVue.configs["flat/essential"],
	vueTsConfigs.recommended,
	...pluginOxlint.configs["flat/recommended"],
	...await atfConfig,
);
