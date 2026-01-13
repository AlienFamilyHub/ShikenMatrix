<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onMounted, onUnmounted } from "vue";

import { useLogsStore } from "./stores/logsStore";

const logsStore = useLogsStore();
let unlisten: Promise<() => void>;

onMounted(() => {
	invoke("start");
	unlisten = listen("log-event", (event) => {
		const processedPayload = (event.payload as string).replace(/data:image\/[^;]+;base64,[a-zA-Z0-9+/]+=*/g, "[BASE64_IMAGE]");
		logsStore.addLog(processedPayload);
	});
});

onUnmounted(() => {
	unlisten.then(fn => fn());
});
</script>

<template>
	<div id="app">
		<router-view />
	</div>
</template>
