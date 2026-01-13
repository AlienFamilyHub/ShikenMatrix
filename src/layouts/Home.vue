<script setup lang="ts">
import { useEventStore } from "@/stores/eventStore";

import { listen } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, ref } from "vue";

const eventStore = useEventStore();
const eventData = computed(() => eventStore.eventData);
const isLoading = ref(true);

onMounted(async () => {
	// 检查eventStore中是否已有数据，如果有则立即显示
	if (Object.keys(eventStore.eventData).length > 0) {
		isLoading.value = false;
	}

	const unlisten = await listen("home-event", (event) => {
		eventStore.setEventData(event.payload as ReturnData);
		isLoading.value = false;
	});

	onUnmounted(() => {
		if (unlisten) {
			unlisten();
		}
	});
});

function formatTimestamp(timestamp: number) {
	const date = new Date(timestamp * 1000);
	return date.toLocaleString();
}
</script>

<template>
	<div class="p-6 bg-gray-100 flex min-h-screen items-center justify-center dark:bg-gray-900">
		<div class="rounded-lg bg-white max-w-2xl w-full shadow-lg overflow-hidden dark:bg-gray-800">
			<div v-if="isLoading" class="p-8 space-y-6">
				<div class="rounded-md bg-gray-200 h-8 w-3/4 animate-pulse dark:bg-gray-700" />
				<div class="rounded-md bg-gray-200 h-6 w-1/2 animate-pulse dark:bg-gray-700" />
				<div class="rounded-md bg-gray-200 h-32 w-full animate-pulse dark:bg-gray-700" />
				<div class="flex items-start space-x-6">
					<div class="rounded-md bg-gray-200 h-32 w-32 animate-pulse dark:bg-gray-700" />
					<div class="flex-1 space-y-3">
						<div class="rounded-md bg-gray-200 h-6 w-3/4 animate-pulse dark:bg-gray-700" />
						<div class="rounded-md bg-gray-200 h-4 w-1/2 animate-pulse dark:bg-gray-700" />
						<div class="rounded-md bg-gray-200 h-4 w-2/3 animate-pulse dark:bg-gray-700" />
					</div>
				</div>
			</div>

			<div v-else class="divide-gray-200 divide-y dark:divide-gray-700">
				<!-- Program Info -->
				<div class="p-8 space-y-4">
					<div class="flex items-center space-x-6">
						<img
							v-if="eventData.icon" :src="eventData.icon" alt="Program Icon"
							class="rounded-md size-16"
						>
						<div>
							<h1 class="text-3xl text-gray-900 font-light mb-1 dark:text-gray-100">
								{{ eventData.data.window_name }}
							</h1>
							<p class="text-lg text-gray-600 font-light dark:text-gray-400">
								{{ eventData.data.process.name }}
							</p>
						</div>
					</div>
				</div>

				<!-- Media Info -->
				<div v-if="eventData.data.media" class="p-8 space-y-6">
					<div class="flex items-start space-x-6">
						<img
							v-if="eventData.AlbumThumbnail" :src="eventData.AlbumThumbnail || '/placeholder.svg'"
							alt="Album Thumbnail" class="rounded-md size-32 object-cover"
						>
						<div class="flex-1 space-y-3">
							<h2 class="text-2xl text-gray-900 font-medium dark:text-gray-100">
								{{ eventData.data.media.title }}
							</h2>
							<p v-if="eventData.data.media.artist" class="text-xl text-gray-700 font-light dark:text-gray-300">
								{{ eventData.data.media.artist }}
							</p>
						</div>
					</div>
					<p
						v-if="eventData.data.media.processName"
						class="text-sm text-gray-400 tracking-wide uppercase dark:text-gray-500"
					>
						{{ eventData.data.media.processName }}
					</p>
				</div>

				<!-- Timestamp -->
				<div class="p-4 bg-gray-50 dark:bg-gray-900">
					<p class="text-sm text-gray-400 dark:text-gray-500">
						更新时间: {{ formatTimestamp(eventData.data.timestamp) }}
					</p>
				</div>
			</div>
		</div>
	</div>
</template>
