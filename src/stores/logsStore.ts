import { defineStore } from "pinia";

const MAX_LOGS = 50; // 限制最大日志数量

export const useLogsStore = defineStore("logs", {
	state: () => ({
		logs: [] as string[],
	}),
	actions: {
		addLog(log: string) {
			this.logs.push(log);
			// 当日志数量超过限制时，移除最旧的日志
			if (this.logs.length > MAX_LOGS) {
				this.logs = this.logs.slice(-MAX_LOGS);
			}
		},
		setLogs(newLogs: string[]) {
			// 确保设置的日志数量不超过限制
			this.logs = newLogs.slice(-MAX_LOGS);
		},
		clearLogs() {
			this.logs = [];
		},
	},
});
