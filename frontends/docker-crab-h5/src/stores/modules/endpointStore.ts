import { defineStore } from "pinia";
import { EndpointInfo, EndpointType } from "../../types/docker";
import {
    getEndpoints,
    createEndpoint,
    updateEndpoint,
    deleteEndpoint,
    testEndpointConnection,
} from "../../api/docker";

export const useEndpointStore = defineStore("endpoint", {
    state: () => ({
        endpoints: [] as EndpointInfo[],
        loading: false,
        error: null as string | null,
        selectedEndpoint: null as EndpointInfo | null,
    }),

    getters: {
        getEndpointById: (state) => (id: string) => {
            return state.endpoints.find((endpoint) => endpoint.config.id === id);
        },
        localEndpoints: (state) => {
            return state.endpoints.filter(
                (endpoint) => endpoint.config.endpoint_type === EndpointType.Local,
            );
        },
        remoteEndpoints: (state) => {
            return state.endpoints.filter(
                (endpoint) => endpoint.config.endpoint_type === EndpointType.Remote,
            );
        },
        cloudEndpoints: (state) => {
            return state.endpoints.filter(
                (endpoint) => endpoint.config.endpoint_type === EndpointType.Cloud,
            );
        },
    },

    actions: {
        async fetchEndpoints() {
            this.loading = true;
            this.error = null;
            try {
                this.endpoints = await getEndpoints();
            } catch (err) {
                this.error = err instanceof Error ? err.message : "Failed to fetch endpoints";
            } finally {
                this.loading = false;
            }
        },

        async addEndpoint(endpointData: Omit<EndpointInfo["config"], "id">) {
            this.loading = true;
            this.error = null;
            try {
                const newEndpoint = await createEndpoint(endpointData);
                this.endpoints.push(newEndpoint);
                return newEndpoint;
            } catch (err) {
                this.error = err instanceof Error ? err.message : "Failed to create endpoint";
                throw err;
            } finally {
                this.loading = false;
            }
        },

        async updateEndpoint(id: string, endpointData: Partial<EndpointInfo["config"]>) {
            this.loading = true;
            this.error = null;
            try {
                const updatedEndpoint = await updateEndpoint(id, endpointData);
                const index = this.endpoints.findIndex((endpoint) => endpoint.config.id === id);
                if (index !== -1) {
                    this.endpoints[index] = updatedEndpoint;
                }
                return updatedEndpoint;
            } catch (err) {
                this.error = err instanceof Error ? err.message : "Failed to update endpoint";
                throw err;
            } finally {
                this.loading = false;
            }
        },

        async removeEndpoint(id: string) {
            this.loading = true;
            this.error = null;
            try {
                await deleteEndpoint(id);
                this.endpoints = this.endpoints.filter((endpoint) => endpoint.config.id !== id);
            } catch (err) {
                this.error = err instanceof Error ? err.message : "Failed to delete endpoint";
                throw err;
            } finally {
                this.loading = false;
            }
        },

        async testConnection(id: string) {
            this.loading = true;
            this.error = null;
            try {
                const status = await testEndpointConnection(id);
                const index = this.endpoints.findIndex((endpoint) => endpoint.config.id === id);
                if (index !== -1) {
                    this.endpoints[index].status = status as any;
                    this.endpoints[index].last_connected_at = new Date().toISOString();
                }
                return status;
            } catch (err) {
                this.error = err instanceof Error ? err.message : "Failed to test connection";
                throw err;
            } finally {
                this.loading = false;
            }
        },

        selectEndpoint(endpoint: EndpointInfo | null) {
            this.selectedEndpoint = endpoint;
        },
    },
});
