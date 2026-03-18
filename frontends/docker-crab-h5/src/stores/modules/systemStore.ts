import { defineStore } from "pinia";
import { DockerSystemStatus } from "@/types";

export const useSystemStore = defineStore("system", {
    state: () => ({
        systemStatus: null as DockerSystemStatus | null,
        isLoading: false,
    }),
    actions: {
        setSystemStatus(status: DockerSystemStatus | null) {
            this.systemStatus = status;
        },
        setIsLoading(loading: boolean) {
            this.isLoading = loading;
        },
    },
});
