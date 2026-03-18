import { defineStore } from "pinia";
import { Container } from "@/types";

export const useContainerStore = defineStore("container", {
    state: () => ({
        containers: [] as Container[],
        selectedContainer: null as Container | null,
        selectedContainerIds: [] as string[],
        containerLogs: [] as string[],
        terminalOutput: [] as string[],
        isLoading: false,
        isLoadingLogs: false,
        isPerformingBatchOperation: false,
        batchOperationResult: { success: 0, failed: 0, errors: [] as string[] },
        authError: "",
        authToken: "",
        isAuthenticated: false,
    }),
    getters: {
        getContainerById: (state) => (id: string) => {
            return state.containers.find((container) => container.id === id);
        },
        isContainerSelected: (state) => (id: string) => {
            return state.selectedContainerIds.includes(id);
        },
        selectedContainersCount: (state) => {
            return state.selectedContainerIds.length;
        },
    },
    actions: {
        setContainers(containers: Container[]) {
            this.containers = containers;
        },
        setSelectedContainer(container: Container | null) {
            this.selectedContainer = container;
        },
        setContainerLogs(logs: string[]) {
            this.containerLogs = logs;
        },
        setTerminalOutput(output: string[]) {
            this.terminalOutput = output;
        },
        setIsLoading(loading: boolean) {
            this.isLoading = loading;
        },
        setIsLoadingLogs(loading: boolean) {
            this.isLoadingLogs = loading;
        },
        setIsPerformingBatchOperation(performing: boolean) {
            this.isPerformingBatchOperation = performing;
        },
        setBatchOperationResult(result: { success: number; failed: number; errors: string[] }) {
            this.batchOperationResult = result;
        },
        setAuthError(error: string) {
            this.authError = error;
        },
        setAuthToken(token: string) {
            this.authToken = token;
        },
        setIsAuthenticated(authenticated: boolean) {
            this.isAuthenticated = authenticated;
        },
        addContainer(container: Container) {
            this.containers.push(container);
        },
        removeContainer(id: string) {
            this.containers = this.containers.filter((container) => container.id !== id);
            this.selectedContainerIds = this.selectedContainerIds.filter(
                (selectedId) => selectedId !== id,
            );
            if (this.selectedContainer && this.selectedContainer.id === id) {
                this.selectedContainer = null;
            }
        },
        updateContainer(updatedContainer: Container) {
            const index = this.containers.findIndex(
                (container) => container.id === updatedContainer.id,
            );
            if (index !== -1) {
                this.containers[index] = updatedContainer;
                if (this.selectedContainer && this.selectedContainer.id === updatedContainer.id) {
                    this.selectedContainer = updatedContainer;
                }
            }
        },
        toggleContainerSelection(id: string) {
            const index = this.selectedContainerIds.indexOf(id);
            if (index === -1) {
                this.selectedContainerIds.push(id);
            } else {
                this.selectedContainerIds.splice(index, 1);
            }
        },
        selectAllContainers() {
            this.selectedContainerIds = this.containers.map((container) => container.id);
        },
        clearContainerSelection() {
            this.selectedContainerIds = [];
        },
        resetBatchOperationResult() {
            this.batchOperationResult = { success: 0, failed: 0, errors: [] };
        },
    },
});
