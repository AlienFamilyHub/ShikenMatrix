import antfu from "@antfu/eslint-config";

export default antfu({
  formatters: true,
  react: true,
  ignores: [
    "**/dist/**",
    "**/build/**",
    "**/target/**",
    "**/gen/**",
    "**/.turbo/**",
    "**/node_modules/**",
  ],
  stylistic: {
    indent: 2,
    quotes: "double",
    semi: true,
  },
  rules: {
    "semi": ["warn", "always"],
    "style/brace-style": ["error", "1tbs", { allowSingleLine: true }],
  },
});
