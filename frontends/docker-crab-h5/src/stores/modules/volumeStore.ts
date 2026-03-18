import { defineStore } from "pinia";
import { Volume } from "@/types";

export const useVolumeStore = defineStore("volume", {
    state: () => ({
        volumes: [] as Volume[],
        selectedVolume: null as Volume | null,
        isLoading: false,
    }),
    getters: {
        getVolumeById: (state) => (id: string) => {
            return state.volumes.find((volume) => volume.id === id);
        },
    },
    actions: {
        setVolumes(volumes: Volume[]) {
            this.volumes = volumes;
        },
        setSelectedVolume(volume: Volume | null) {
            this.selectedVolume = volume;
        },
        setIsLoading(loading: boolean) {
            this.isLoading = loading;
        },
        addVolume(volume: Volume) {
            this.volumes.push(volume);
        },
        removeVolume(id: string) {
            this.volumes = this.volumes.filter((volume) => volume.id !== id);
            if (this.selectedVolume && this.selectedVolume.id === id) {
                this.selectedVolume = null;
            }
        },
    },
});
