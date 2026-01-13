import { defineStore } from "pinia";

export const useEventStore = defineStore("event", {
	state: () => ({
		eventData: JSON.parse(localStorage.getItem("eventData") || "{}") as ReturnData,
	}),
	actions: {
		setEventData(data: ReturnData) {
			this.eventData = data;
			localStorage.setItem("eventData", JSON.stringify(data));
		},
	},
});
