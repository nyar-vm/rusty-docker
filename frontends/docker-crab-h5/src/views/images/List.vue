<template>
  <div class="space-y-5">
    <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
      <h2 class="text-xl font-semibold text-white">镜像</h2>
      <div class="flex flex-col sm:flex-row gap-2">
        <button class="px-4 py-2.5 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all flex items-center justify-center gap-2 text-sm">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          拉取镜像
        </button>
        <button @click="listImages" class="px-4 py-2.5 bg-white/5 border border-white/10 rounded-lg font-medium hover:bg-white/10 transition-all flex items-center justify-center gap-2 text-sm">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          刷新
        </button>
      </div>
    </div>
    
    <!-- 拉取镜像卡片 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
      <h3 class="text-base font-semibold text-white mb-3">拉取镜像</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <input v-model="newImageName" placeholder="镜像名称 (如 nginx:latest)" class="px-4 py-2.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] transition-all">
        <button @click="pullImage" :disabled="isPullingImage" class="px-4 py-2.5 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all flex items-center justify-center gap-2 text-sm">
          {{ isPullingImage ? '拉取中...' : '拉取镜像' }}
        </button>
      </div>
    </div>
    
    <!-- 批量操作卡片 -->
    <div v-if="selectedImagesCount > 0" class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
      <h3 class="text-base font-semibold text-white mb-3">批量操作 (已选择 {{ selectedImagesCount }} 个镜像)</h3>
      <div class="flex flex-wrap gap-2">
        <button @click="batchDeleteImages" :disabled="isPerformingBatchOperation" class="px-4 py-2 bg-red-600/10 border border-red-600/20 text-red-400 rounded hover:bg-red-600/20 transition-all text-sm">
          批量删除
        </button>
        <button @click="clearImageSelection" class="px-4 py-2 bg-white/5 border border-white/10 text-white/70 rounded hover:bg-white/10 transition-all text-sm">
          取消选择
        </button>
      </div>
      
      <!-- 批量操作结果 -->
      <div v-if="showBatchResult" class="mt-3 p-3 bg-white/5 rounded-lg">
        <div class="text-sm text-white">
          <div class="flex items-center gap-2 mb-1">
            <span class="text-green-400">成功: {{ batchOperationResult.success }}</span>
            <span class="text-red-400">失败: {{ batchOperationResult.failed }}</span>
          </div>
          <div v-if="batchOperationResult.errors.length > 0" class="mt-2 text-red-400 text-xs">
            <div v-for="(error, index) in batchOperationResult.errors" :key="index" class="mb-1">
              {{ error }}
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 镜像列表 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-base font-semibold text-white">镜像列表</h3>
        <button v-if="images.length > 0" @click="selectAllImages" class="text-sm text-[#0db7ed] hover:underline">
          全选
        </button>
      </div>
      
      <div v-if="images.length === 0" class="text-center py-8 text-white/40">
        暂无镜像
      </div>
      
      <div v-else class="space-y-2">
        <div v-for="image in images" :key="image.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-all">
          <div class="flex-1 mb-2 sm:mb-0">
            <div class="flex items-center gap-2">
              <input 
                type="checkbox" 
                :checked="isImageSelected(image.id)" 
                @click.stop="toggleImageSelection(image.id)"
                class="w-4 h-4 rounded border-white/20 bg-white/5 text-[#0db7ed] focus:ring-[#0db7ed]"
              >
              <div class="w-8 h-8 bg-cyan-600/20 rounded-lg flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-cyan-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
                </svg>
              </div>
              <div @click="showImageDetails(image)">
                <h4 class="font-medium text-white text-sm">{{ image.repoTags?.[0] || '无标签' }}</h4>
                <p class="text-xs text-white/50">{{ image.id.substring(0, 12) }}</p>
              </div>
            </div>
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-center gap-2">
            <span class="px-2 py-1 bg-white/5 border border-white/10 text-white/70 rounded text-xs font-medium">
              {{ image.size }}
            </span>
            <button @click.stop="deleteImage(image.id)" class="px-2 py-1 bg-red-600/10 border border-red-600/20 text-red-400 rounded hover:bg-red-600/20 transition-all text-xs">
              删除
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useImageStore } from "@/stores/modules/imageStore";
import { imageApi } from "@/api/docker";

const imageStore = useImageStore();
const { images, isPullingImage, isPerformingBatchOperation, batchOperationResult } = imageStore;

const newImageName = ref("");
const showBatchResult = ref(false);

const selectedImagesCount = computed(() => imageStore.selectedImagesCount);
const isImageSelected = computed(() => imageStore.isImageSelected);

const listImages = async () => {
    imageStore.setIsLoading(true);
    try {
        const data = await imageApi.list();
        imageStore.setImages(data);
    } catch (error) {
        console.error("Failed to list images:", error);
    } finally {
        imageStore.setIsLoading(false);
    }
};

const pullImage = async () => {
    if (!newImageName.value) return;

    imageStore.setIsPullingImage(true);
    try {
        const image = await imageApi.pull(newImageName.value);
        imageStore.addImage(image);
        newImageName.value = "";
    } catch (error) {
        console.error("Failed to pull image:", error);
    } finally {
        imageStore.setIsPullingImage(false);
    }
};

const deleteImage = async (id: string) => {
    try {
        await imageApi.remove(id);
        imageStore.removeImage(id);
    } catch (error) {
        console.error("Failed to delete image:", error);
    }
};

const showImageDetails = (image: any) => {
    imageStore.setSelectedImage(image);
    // 这里可以通过路由或状态管理来切换到详情视图
};

const toggleImageSelection = (id: string) => {
    imageStore.toggleImageSelection(id);
    showBatchResult.value = false;
};

const selectAllImages = () => {
    imageStore.selectAllImages();
    showBatchResult.value = false;
};

const clearImageSelection = () => {
    imageStore.clearImageSelection();
    showBatchResult.value = false;
};

const batchDeleteImages = async () => {
    const selectedIds = imageStore.selectedImageIds;
    if (selectedIds.length === 0) return;

    if (!confirm(`确定要删除选中的 ${selectedIds.length} 个镜像吗？`)) {
        return;
    }

    imageStore.setIsPerformingBatchOperation(true);
    imageStore.resetBatchOperationResult();
    showBatchResult.value = false;

    try {
        await imageApi.batchRemove(selectedIds);
        imageStore.setBatchOperationResult({ success: selectedIds.length, failed: 0, errors: [] });
        imageStore.clearImageSelection();
        await listImages();
    } catch (error) {
        console.error("Failed to batch delete images:", error);
        imageStore.setBatchOperationResult({
            success: 0,
            failed: selectedIds.length,
            errors: [error instanceof Error ? error.message : "批量删除失败"],
        });
    } finally {
        imageStore.setIsPerformingBatchOperation(false);
        showBatchResult.value = true;
    }
};

onMounted(() => {
    listImages();
});
</script>
