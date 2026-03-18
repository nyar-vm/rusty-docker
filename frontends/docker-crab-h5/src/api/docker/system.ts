import apiClient from "..";
import { DockerSystemStatus } from "@/types";

export const systemApi = {
    status: (): Promise<DockerSystemStatus> => {
        return apiClient.get("/system/status");
    },
    restart: (): Promise<void> => {
        return apiClient.post("/system/restart");
    },
};
