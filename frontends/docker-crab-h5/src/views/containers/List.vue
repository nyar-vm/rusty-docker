<template>
  <div class="space-y-5">
    <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
      <h2 class="text-xl font-semibold text-white">容器</h2>
      <div class="flex flex-col sm:flex-row gap-2">
        <button class="px-4 py-2.5 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all flex items-center justify-center gap-2 text-sm">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          运行新容器
        </button>
        <button @click="listContainers" class="px-4 py-2.5 bg-white/5 border border-white/10 rounded-lg font-medium hover:bg-white/10 transition-all flex items-center justify-center gap-2 text-sm">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          刷新
        </button>
      </div>
    </div>
    
    <!-- 容器操作卡片 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
      <h3 class="text-base font-semibold text-white mb-3">创建容器</h3>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
        <input v-model="newContainerImage" placeholder="镜像名称 (如 nginx:latest)" class="px-4 py-2.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] transition-all">
        <input v-model="newContainerName" placeholder="容器名称" class="px-4 py-2.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] transition-all">
        <input v-model="newContainerPorts" placeholder="端口映射 (如 8080:80)" class="px-4 py-2.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] transition-all">
      </div>
      <div class="mt-3">
        <button @click="runContainer" class="px-4 py-2.5 bg-gradient-to-r from-green-600 to-emerald-700 rounded-lg font-medium hover:shadow-lg transition-all text-sm">
          运行容器
        </button>
      </div>
    </div>
    
    <!-- 批量操作卡片 -->
    <div v-if="selectedContainersCount > 0" class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
      <h3 class="text-base font-semibold text-white mb-3">批量操作 (已选择 {{ selectedContainersCount }} 个容器)</h3>
      <div class="flex flex-wrap gap-2">
        <button @click="batchStartContainers" :disabled="isPerformingBatchOperation" class="px-4 py-2 bg-green-600/10 border border-green-600/20 text-green-400 rounded hover:bg-green-600/20 transition-all text-sm">
          批量启动
        </button>
        <button @click="batchStopContainers" :disabled="isPerformingBatchOperation" class="px-4 py-2 bg-yellow-600/10 border border-yellow-600/20 text-yellow-400 rounded hover:bg-yellow-600/20 transition-all text-sm">
          批量停止
        </button>
        <button @click="batchRestartContainers" :disabled="isPerformingBatchOperation" class="px-4 py-2 bg-[#0db7ed]/10 border border-[#0db7ed]/20 text-[#0db7ed] rounded hover:bg-[#0db7ed]/20 transition-all text-sm">
          批量重启
        </button>
        <button @click="batchDeleteContainers" :disabled="isPerformingBatchOperation" class="px-4 py-2 bg-red-600/10 border border-red-600/20 text-red-400 rounded hover:bg-red-600/20 transition-all text-sm">
          批量删除
        </button>
        <button @click="clearContainerSelection" class="px-4 py-2 bg-white/5 border border-white/10 text-white/70 rounded hover:bg-white/10 transition-all text-sm">
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
    
    <!-- 容器列表 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-base font-semibold text-white">容器列表</h3>
        <button v-if="containers.length > 0" @click="selectAllContainers" class="text-sm text-[#0db7ed] hover:underline">
          全选
        </button>
      </div>
      
      <div v-if="containers.length === 0" class="text-center py-8 text-white/40">
        暂无容器
      </div>
      
      <div v-else class="space-y-2">
        <div v-for="container in containers" :key="container.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-all">
          <div class="flex-1 mb-2 sm:mb-0">
            <div class="flex items-center gap-2">
              <input 
                type="checkbox" 
                :checked="isContainerSelected(container.id)" 
                @click.stop="toggleContainerSelection(container.id)"
                class="w-4 h-4 rounded border-white/20 bg-white/5 text-[#0db7ed] focus:ring-[#0db7ed]"
              >
              <div class="w-8 h-8 bg-[#0db7ed]/20 rounded-lg flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[#0db7ed]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                </svg>
              </div>
              <div @click="showContainerDetails(container)">
                <h4 class="font-medium text-white text-sm">{{ container.name }}</h4>
                <p class="text-xs text-white/50">{{ container.image }}</p>
              </div>
            </div>
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-center gap-2">
            <span :class="container.status === 'Running' ? 'px-2 py-1 bg-green-600/10 text-green-400 rounded text-xs font-medium border border-green-600/20' : 'px-2 py-1 bg-gray-600/10 text-gray-400 rounded text-xs font-medium border border-gray-600/20'">
              {{ container.status }}
            </span>
            <div class="flex gap-1">
              <button @click.stop="startContainer(container.id)" v-if="container.status !== 'Running'" class="px-2 py-1 bg-green-600/10 border border-green-600/20 text-green-400 rounded hover:bg-green-600/20 transition-all text-xs">
                启动
              </button>
              <button @click.stop="stopContainer(container.id)" v-if="container.status === 'Running'" class="px-2 py-1 bg-yellow-600/10 border border-yellow-600/20 text-yellow-400 rounded hover:bg-yellow-600/20 transition-all text-xs">
                停止
              </button>
              <button @click.stop="restartContainer(container.id)" class="px-2 py-1 bg-[#0db7ed]/10 border border-[#0db7ed]/20 text-[#0db7ed] rounded hover:bg-[#0db7ed]/20 transition-all text-xs">
                重启
              </button>
              <button @click.stop="deleteContainer(container.id)" class="px-2 py-1 bg-red-600/10 border border-red-600/20 text-red-400 rounded hover:bg-red-600/20 transition-all text-xs">
                删除
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useContainerStore } from "@/stores/modules/containerStore";
import { containerApi } from "@/api/docker";

const containerStore = useContainerStore();
const { containers, selectedContainerIds, isPerformingBatchOperation, batchOperationResult } =
    containerStore;

const newContainerImage = ref("");
const newContainerName = ref("");
const newContainerPorts = ref("");
const showBatchResult = ref(false);

const selectedContainersCount = computed(() => containerStore.selectedContainersCount);
const isContainerSelected = computed(() => containerStore.isContainerSelected);

const listContainers = async () => {
    containerStore.setIsLoading(true);
    try {
        const data = await containerApi.list();
        containerStore.setContainers(data);
    } catch (error) {
        console.error("Failed to list containers:", error);
    } finally {
        containerStore.setIsLoading(false);
    }
};

const runContainer = async () => {
    if (!newContainerImage.value) return;

    containerStore.setIsLoading(true);
    try {
        const container = await containerApi.create({
            image: newContainerImage.value,
            name: newContainerName.value,
            ports: newContainerPorts.value,
        });
        containerStore.addContainer(container);
        newContainerImage.value = "";
        newContainerName.value = "";
        newContainerPorts.value = "";
    } catch (error) {
        console.error("Failed to run container:", error);
    } finally {
        containerStore.setIsLoading(false);
    }
};

const startContainer = async (id: string) => {
    try {
        await containerApi.start(id);
        const container = await containerApi.get(id);
        containerStore.updateContainer(container);
    } catch (error) {
        console.error("Failed to start container:", error);
    }
};

const stopContainer = async (id: string) => {
    try {
        await containerApi.stop(id);
        const container = await containerApi.get(id);
        containerStore.updateContainer(container);
    } catch (error) {
        console.error("Failed to stop container:", error);
    }
};

const restartContainer = async (id: string) => {
    try {
        await containerApi.restart(id);
        const container = await containerApi.get(id);
        containerStore.updateContainer(container);
    } catch (error) {
        console.error("Failed to restart container:", error);
    }
};

const deleteContainer = async (id: string) => {
    try {
        await containerApi.remove(id);
        containerStore.removeContainer(id);
    } catch (error) {
        console.error("Failed to delete container:", error);
    }
};

const showContainerDetails = (container: any) => {
    containerStore.setSelectedContainer(container);
    // 这里可以通过路由或状态管理来切换到详情视图
};

const toggleContainerSelection = (id: string) => {
    containerStore.toggleContainerSelection(id);
    showBatchResult.value = false;
};

const selectAllContainers = () => {
    containerStore.selectAllContainers();
    showBatchResult.value = false;
};

const clearContainerSelection = () => {
    containerStore.clearContainerSelection();
    showBatchResult.value = false;
};

const batchStartContainers = async () => {
    if (selectedContainerIds.length === 0) return;

    containerStore.setIsPerformingBatchOperation(true);
    containerStore.resetBatchOperationResult();
    showBatchResult.value = false;

    try {
        await containerApi.batchStart(selectedContainerIds);
        containerStore.setBatchOperationResult({
            success: selectedContainerIds.length,
            failed: 0,
            errors: [],
        });
        await listContainers();
    } catch (error) {
        console.error("Failed to batch start containers:", error);
        containerStore.setBatchOperationResult({
            success: 0,
            failed: selectedContainerIds.length,
            errors: [error instanceof Error ? error.message : "批量启动失败"],
        });
    } finally {
        containerStore.setIsPerformingBatchOperation(false);
        showBatchResult.value = true;
    }
};

const batchStopContainers = async () => {
    if (selectedContainerIds.length === 0) return;

    containerStore.setIsPerformingBatchOperation(true);
    containerStore.resetBatchOperationResult();
    showBatchResult.value = false;

    try {
        await containerApi.batchStop(selectedContainerIds);
        containerStore.setBatchOperationResult({
            success: selectedContainerIds.length,
            failed: 0,
            errors: [],
        });
        await listContainers();
    } catch (error) {
        console.error("Failed to batch stop containers:", error);
        containerStore.setBatchOperationResult({
            success: 0,
            failed: selectedContainerIds.length,
            errors: [error instanceof Error ? error.message : "批量停止失败"],
        });
    } finally {
        containerStore.setIsPerformingBatchOperation(false);
        showBatchResult.value = true;
    }
};

const batchRestartContainers = async () => {
    if (selectedContainerIds.length === 0) return;

    containerStore.setIsPerformingBatchOperation(true);
    containerStore.resetBatchOperationResult();
    showBatchResult.value = false;

    try {
        await containerApi.batchRestart(selectedContainerIds);
        containerStore.setBatchOperationResult({
            success: selectedContainerIds.length,
            failed: 0,
            errors: [],
        });
        await listContainers();
    } catch (error) {
        console.error("Failed to batch restart containers:", error);
        containerStore.setBatchOperationResult({
            success: 0,
            failed: selectedContainerIds.length,
            errors: [error instanceof Error ? error.message : "批量重启失败"],
        });
    } finally {
        containerStore.setIsPerformingBatchOperation(false);
        showBatchResult.value = true;
    }
};

const batchDeleteContainers = async () => {
    if (selectedContainerIds.length === 0) return;

    if (!confirm(`确定要删除选中的 ${selectedContainerIds.length} 个容器吗？`)) {
        return;
    }

    containerStore.setIsPerformingBatchOperation(true);
    containerStore.resetBatchOperationResult();
    showBatchResult.value = false;

    try {
        await containerApi.batchRemove(selectedContainerIds);
        containerStore.setBatchOperationResult({
            success: selectedContainerIds.length,
            failed: 0,
            errors: [],
        });
        containerStore.clearContainerSelection();
        await listContainers();
    } catch (error) {
        console.error("Failed to batch delete containers:", error);
        containerStore.setBatchOperationResult({
            success: 0,
            failed: selectedContainerIds.length,
            errors: [error instanceof Error ? error.message : "批量删除失败"],
        });
    } finally {
        containerStore.setIsPerformingBatchOperation(false);
        showBatchResult.value = true;
    }
};

onMounted(() => {
    listContainers();
});
</script>
