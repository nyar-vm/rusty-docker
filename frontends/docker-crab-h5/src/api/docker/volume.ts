import apiClient from "..";
import { Volume } from "@/types";

export const volumeApi = {
    list: (): Promise<Volume[]> => {
        return apiClient.get("/volumes");
    },
    get: (id: string): Promise<Volume> => {
        return apiClient.get(`/volumes/${id}`);
    },
    create: (data: {
        name: string;
        driver: string;
    }): Promise<Volume> => {
        return apiClient.post("/volumes", data);
    },
    remove: (id: string): Promise<void> => {
        return apiClient.delete(`/volumes/${id}`);
    },
};
