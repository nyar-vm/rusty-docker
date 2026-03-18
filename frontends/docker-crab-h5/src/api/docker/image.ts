import apiClient from "..";
import { Image } from "@/types";

export const imageApi = {
    list: (): Promise<Image[]> => {
        return apiClient.get("/images");
    },
    get: (id: string): Promise<Image> => {
        return apiClient.get(`/images/${id}`);
    },
    pull: (name: string): Promise<Image> => {
        return apiClient.post("/images/pull", { name });
    },
    remove: (id: string): Promise<void> => {
        return apiClient.delete(`/images/${id}`);
    },
    batchRemove: (ids: string[]): Promise<void> => {
        return apiClient.post("/images/batch/remove", { ids });
    },
};
