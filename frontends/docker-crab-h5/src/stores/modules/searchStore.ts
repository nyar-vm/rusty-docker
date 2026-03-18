import { defineStore } from "pinia";
import { Container, Image, Network, Volume } from "@/types";
import { useTagStore } from "./tagStore";

export const useSearchStore = defineStore("search", {
    state: () => ({
        searchQuery: "",
        selectedTags: [] as string[],
        resourceType: "all" as "all" | "containers" | "images" | "networks" | "volumes",
    }),
    getters: {
        filteredContainers: (state) => (containers: Container[]) => {
            const tagStore = useTagStore();
            return containers.filter((container) => {
                // Filter by search query
                const matchesQuery =
                    !state.searchQuery ||
                    container.name.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
                    container.image.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
                    container.id.toLowerCase().includes(state.searchQuery.toLowerCase());

                // Filter by tags
                const containerTags = tagStore.getTagsByResourceId(container.id);
                const matchesTags =
                    state.selectedTags.length === 0 ||
                    state.selectedTags.every((tag) => containerTags.includes(tag));

                return matchesQuery && matchesTags;
            });
        },
        filteredImages: (state) => (images: Image[]) => {
            const tagStore = useTagStore();
            return images.filter((image) => {
                // Filter by search query
                const matchesQuery =
                    !state.searchQuery ||
                    (image.repoTags &&
                        image.repoTags.some((tag) =>
                            tag.toLowerCase().includes(state.searchQuery.toLowerCase()),
                        )) ||
                    image.id.toLowerCase().includes(state.searchQuery.toLowerCase());

                // Filter by tags
                const imageTags = tagStore.getTagsByResourceId(image.id);
                const matchesTags =
                    state.selectedTags.length === 0 ||
                    state.selectedTags.every((tag) => imageTags.includes(tag));

                return matchesQuery && matchesTags;
            });
        },
        filteredNetworks: (state) => (networks: Network[]) => {
            const tagStore = useTagStore();
            return networks.filter((network) => {
                // Filter by search query
                const matchesQuery =
                    !state.searchQuery ||
                    network.name.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
                    network.id.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
                    network.driver.toLowerCase().includes(state.searchQuery.toLowerCase());

                // Filter by tags
                const networkTags = tagStore.getTagsByResourceId(network.id);
                const matchesTags =
                    state.selectedTags.length === 0 ||
                    state.selectedTags.every((tag) => networkTags.includes(tag));

                return matchesQuery && matchesTags;
            });
        },
        filteredVolumes: (state) => (volumes: Volume[]) => {
            const tagStore = useTagStore();
            return volumes.filter((volume) => {
                // Filter by search query
                const matchesQuery =
                    !state.searchQuery ||
                    volume.name.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
                    volume.id.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
                    volume.mount_point.toLowerCase().includes(state.searchQuery.toLowerCase());

                // Filter by tags
                const volumeTags = tagStore.getTagsByResourceId(volume.id);
                const matchesTags =
                    state.selectedTags.length === 0 ||
                    state.selectedTags.every((tag) => volumeTags.includes(tag));

                return matchesQuery && matchesTags;
            });
        },
    },
    actions: {
        setSearchQuery(query: string) {
            this.searchQuery = query;
        },
        setSelectedTags(tags: string[]) {
            this.selectedTags = tags;
        },
        toggleTag(tag: string) {
            const index = this.selectedTags.indexOf(tag);
            if (index === -1) {
                this.selectedTags.push(tag);
            } else {
                this.selectedTags.splice(index, 1);
            }
        },
        clearSearch() {
            this.searchQuery = "";
            this.selectedTags = [];
        },
        setResourceType(type: "all" | "containers" | "images" | "networks" | "volumes") {
            this.resourceType = type;
        },
    },
});
