import apiClient from "..";
import { Network } from "@/types";

export const networkApi = {
    list: (): Promise<Network[]> => {
        return apiClient.get("/networks");
    },
    get: (id: string): Promise<Network> => {
        return apiClient.get(`/networks/${id}`);
    },
    create: (data: {
        name: string;
        driver: string;
    }): Promise<Network> => {
        return apiClient.post("/networks", data);
    },
    remove: (id: string): Promise<void> => {
        return apiClient.delete(`/networks/${id}`);
    },
};
