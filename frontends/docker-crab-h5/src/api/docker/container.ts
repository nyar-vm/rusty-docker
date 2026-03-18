import apiClient from "..";
import { Container } from "@/types";

export const containerApi = {
    list: (): Promise<Container[]> => {
        return apiClient.get("/containers");
    },
    get: (id: string): Promise<Container> => {
        return apiClient.get(`/containers/${id}`);
    },
    create: (data: {
        image: string;
        name: string;
        ports?: string;
    }): Promise<Container> => {
        return apiClient.post("/containers", data);
    },
    start: (id: string): Promise<void> => {
        return apiClient.post(`/containers/${id}/start`);
    },
    stop: (id: string): Promise<void> => {
        return apiClient.post(`/containers/${id}/stop`);
    },
    restart: (id: string): Promise<void> => {
        return apiClient.post(`/containers/${id}/restart`);
    },
    remove: (id: string): Promise<void> => {
        return apiClient.delete(`/containers/${id}`);
    },
    logs: (id: string): Promise<string[]> => {
        return apiClient.get(`/containers/${id}/logs`);
    },
    exec: (id: string, command: string): Promise<string[]> => {
        return apiClient.post(`/containers/${id}/exec`, { command });
    },
    batchStart: (ids: string[]): Promise<void> => {
        return apiClient.post("/containers/batch/start", { ids });
    },
    batchStop: (ids: string[]): Promise<void> => {
        return apiClient.post("/containers/batch/stop", { ids });
    },
    batchRestart: (ids: string[]): Promise<void> => {
        return apiClient.post("/containers/batch/restart", { ids });
    },
    batchRemove: (ids: string[]): Promise<void> => {
        return apiClient.post("/containers/batch/remove", { ids });
    },
};
