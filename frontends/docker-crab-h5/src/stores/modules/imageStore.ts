import { defineStore } from "pinia";
import { Image } from "@/types";

export const useImageStore = defineStore("image", {
    state: () => ({
        images: [] as Image[],
        selectedImage: null as Image | null,
        selectedImageIds: [] as string[],
        isLoading: false,
        isPullingImage: false,
        isPerformingBatchOperation: false,
        batchOperationResult: { success: 0, failed: 0, errors: [] as string[] },
    }),
    getters: {
        getImageById: (state) => (id: string) => {
            return state.images.find((image) => image.id === id);
        },
        isImageSelected: (state) => (id: string) => {
            return state.selectedImageIds.includes(id);
        },
        selectedImagesCount: (state) => {
            return state.selectedImageIds.length;
        },
    },
    actions: {
        setImages(images: Image[]) {
            this.images = images;
        },
        setSelectedImage(image: Image | null) {
            this.selectedImage = image;
        },
        setIsLoading(loading: boolean) {
            this.isLoading = loading;
        },
        setIsPullingImage(pulling: boolean) {
            this.isPullingImage = pulling;
        },
        setIsPerformingBatchOperation(performing: boolean) {
            this.isPerformingBatchOperation = performing;
        },
        setBatchOperationResult(result: { success: number; failed: number; errors: string[] }) {
            this.batchOperationResult = result;
        },
        addImage(image: Image) {
            this.images.push(image);
        },
        removeImage(id: string) {
            this.images = this.images.filter((image) => image.id !== id);
            this.selectedImageIds = this.selectedImageIds.filter((selectedId) => selectedId !== id);
            if (this.selectedImage && this.selectedImage.id === id) {
                this.selectedImage = null;
            }
        },
        toggleImageSelection(id: string) {
            const index = this.selectedImageIds.indexOf(id);
            if (index === -1) {
                this.selectedImageIds.push(id);
            } else {
                this.selectedImageIds.splice(index, 1);
            }
        },
        selectAllImages() {
            this.selectedImageIds = this.images.map((image) => image.id);
        },
        clearImageSelection() {
            this.selectedImageIds = [];
        },
        resetBatchOperationResult() {
            this.batchOperationResult = { success: 0, failed: 0, errors: [] };
        },
    },
});
