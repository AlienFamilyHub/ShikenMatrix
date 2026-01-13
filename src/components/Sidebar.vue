<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarHeader,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
} from "@/components/ui/sidebar";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { Icon } from "@iconify/vue";
import { computed } from "vue";
import { useRoute } from "vue-router";

const route = useRoute();
const currentPath = computed(() => route.path);

const navItems = [
	{ name: "首页", path: "/", icon: "mingcute:home-4-line" },
	{ name: "设置", path: "/settings", icon: "mingcute:settings-3-line" },
	{ name: "日志", path: "/logs", icon: "mingcute:file-line" },
];

const currentYear = new Date().getFullYear();
</script>

<template>
	<Sidebar class="border-gray-200 dark:border-gray-800 md:block">
		<SidebarHeader class="p-5">
			<div class="flex items-center space-x-2">
				<div class="bg-primary rounded-md flex h-10 w-10 items-center justify-center">
					<span class="text-primary-foreground text-lg font-bold">MT</span>
				</div>
				<h1 class="text-base font-bold">
					Kizuna @
				</h1>
			</div>
		</SidebarHeader>

		<SidebarContent>
			<Card class="bg-background mx-2 dark:bg-gray-800">
				<CardContent>
					<SidebarMenu>
						<SidebarMenuItem v-for="item in navItems" :key="item.name" class="mt-1">
							<SidebarMenuButton
								:is-active="currentPath === item.path"
								class="text-lg flex h-10 w-full items-center dark:text-gray-300 dark:hover:bg-gray-700"
							>
								<RouterLink :to="item.path" class="flex w-full items-center">
									<Icon :icon="item.icon" class="mr-2 size-6" />
									<span>{{ item.name }}</span>
								</RouterLink>
							</SidebarMenuButton>
						</SidebarMenuItem>
					</SidebarMenu>
				</CardContent>
			</Card>
		</SidebarContent>

		<SidebarFooter>
			<Separator class="dark:bg-gray-700" />
			<div class="p-4">
				<div class="mb-2 flex justify-center">
					<TooltipProvider :delay-duration="100">
						<Tooltip>
							<TooltipTrigger>
								<a href="https://github.com/yourusername/media-tracker" target="_blank" rel="noopener noreferrer">
									<Button variant="ghost" size="icon" class="dark:text-gray-300 dark:hover:bg-gray-700">
										<Icon icon="mingcute:github-line" class="size-5" />
										<span class="sr-only">GitHub Repository</span>
									</Button>
								</a>
							</TooltipTrigger>
							<TooltipContent>
								<p>GitHub Repository</p>
							</TooltipContent>
						</Tooltip>
					</TooltipProvider>
				</div>
				<div class="text-sm text-gray-500 text-center dark:text-gray-400">
					<p>© {{ currentYear }} Media Tracker</p>
					<p class="text-xs">
						Open Source Project
					</p>
				</div>
			</div>
		</SidebarFooter>
	</Sidebar>
</template>
