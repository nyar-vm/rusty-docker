import { defineStore } from "pinia";
import { Network } from "@/types";

export const useNetworkStore = defineStore("network", {
    state: () => ({
        networks: [] as Network[],
        selectedNetwork: null as Network | null,
        isLoading: false,
    }),
    getters: {
        getNetworkById: (state) => (id: string) => {
            return state.networks.find((network) => network.id === id);
        },
    },
    actions: {
        setNetworks(networks: Network[]) {
            this.networks = networks;
        },
        setSelectedNetwork(network: Network | null) {
            this.selectedNetwork = network;
        },
        setIsLoading(loading: boolean) {
            this.isLoading = loading;
        },
        addNetwork(network: Network) {
            this.networks.push(network);
        },
        removeNetwork(id: string) {
            this.networks = this.networks.filter((network) => network.id !== id);
            if (this.selectedNetwork && this.selectedNetwork.id === id) {
                this.selectedNetwork = null;
            }
        },
    },
});
