import { defineStore } from "pinia";

export const useTagStore = defineStore("tag", {
    state: () => ({
        allTags: [] as string[],
        resourceTags: new Map<string, string[]>(), // key: resourceId, value: tags
    }),
    getters: {
        getTagsByResourceId: (state) => (resourceId: string) => {
            return state.resourceTags.get(resourceId) || [];
        },
        getResourcesByTag: (state) => (tag: string) => {
            const resources: string[] = [];
            state.resourceTags.forEach((tags, resourceId) => {
                if (tags.includes(tag)) {
                    resources.push(resourceId);
                }
            });
            return resources;
        },
        getAllUniqueTags: (state) => {
            return state.allTags;
        },
    },
    actions: {
        addTagToResource(resourceId: string, tag: string) {
            const tags = this.resourceTags.get(resourceId) || [];
            if (!tags.includes(tag)) {
                tags.push(tag);
                this.resourceTags.set(resourceId, tags);
                if (!this.allTags.includes(tag)) {
                    this.allTags.push(tag);
                }
            }
        },
        removeTagFromResource(resourceId: string, tag: string) {
            const tags = this.resourceTags.get(resourceId) || [];
            const updatedTags = tags.filter((t) => t !== tag);
            this.resourceTags.set(resourceId, updatedTags);

            // Check if this tag is no longer used by any resource
            let tagUsed = false;
            this.resourceTags.forEach((tags) => {
                if (tags.includes(tag)) {
                    tagUsed = true;
                }
            });
            if (!tagUsed) {
                this.allTags = this.allTags.filter((t) => t !== tag);
            }
        },
        setTagsForResource(resourceId: string, tags: string[]) {
            this.resourceTags.set(resourceId, tags);
            // Update allTags to include any new tags
            tags.forEach((tag) => {
                if (!this.allTags.includes(tag)) {
                    this.allTags.push(tag);
                }
            });
            // Remove tags that are no longer used
            this.cleanupUnusedTags();
        },
        cleanupUnusedTags() {
            const usedTags = new Set<string>();
            this.resourceTags.forEach((tags) => {
                tags.forEach((tag) => usedTags.add(tag));
            });
            this.allTags = Array.from(usedTags);
        },
        removeResource(resourceId: string) {
            this.resourceTags.delete(resourceId);
            this.cleanupUnusedTags();
        },
    },
});
