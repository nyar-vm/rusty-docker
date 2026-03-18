<template>
  <div class="space-y-6">
    <div class="flex items-center gap-4">
      <h2 class="text-xl font-bold text-white">系统状态</h2>
      <button @click="getSystemStatus" class="px-4 py-2 bg-white/10 rounded-lg font-medium hover:bg-white/15 transition-all flex items-center justify-center gap-2">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        刷新
      </button>
    </div>
    
    <div v-if="systemStatus" class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- 系统信息卡片 -->
      <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
        <h3 class="text-lg font-semibold text-white mb-4">系统信息</h3>
        <div class="space-y-3">
          <div class="flex justify-between">
            <span class="text-white/60">状态:</span>
            <span class="text-green-400 font-medium">{{ systemStatus.status }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/60">Docker 版本:</span>
            <span class="text-white">{{ systemStatus.version }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/60">API 版本:</span>
            <span class="text-white">{{ systemStatus.apiVersion }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/60">操作系统:</span>
            <span class="text-white">{{ systemStatus.osType }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/60">架构:</span>
            <span class="text-white">{{ systemStatus.architecture }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/60">内核版本:</span>
            <span class="text-white">{{ systemStatus.kernelVersion }}</span>
          </div>
        </div>
      </div>
      
      <!-- 资源使用卡片 -->
      <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
        <h3 class="text-lg font-semibold text-white mb-4">资源使用</h3>
        <div class="space-y-4">
          <div>
            <div class="flex justify-between mb-1">
              <span class="text-white/60 text-sm">内存使用</span>
              <span class="text-white text-sm">{{ formatSize(systemStatus.usedMemory) }} / {{ formatSize(systemStatus.totalMemory) }}</span>
            </div>
            <div class="w-full bg-white/10 rounded-full h-2">
              <div class="bg-gradient-to-r from-[#0db7ed] to-[#0071c5] h-2 rounded-full" :style="{ width: `${(systemStatus.usedMemory / systemStatus.totalMemory) * 100}%` }"></div>
            </div>
          </div>
          <div>
            <div class="flex justify-between mb-1">
              <span class="text-white/60 text-sm">CPU 使用</span>
              <span class="text-white text-sm">{{ systemStatus.usedCPU }}% / {{ systemStatus.totalCPU }}%</span>
            </div>
            <div class="w-full bg-white/10 rounded-full h-2">
              <div class="bg-gradient-to-r from-green-500 to-emerald-600 h-2 rounded-full" :style="{ width: `${systemStatus.usedCPU}%` }"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <div v-else class="text-center py-10 text-white/60">
      加载系统状态中...
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useSystemStore } from "@/stores/modules/systemStore";
import { systemApi } from "@/api/docker";

const systemStore = useSystemStore();
const { systemStatus } = systemStore;

const getSystemStatus = async () => {
    systemStore.setIsLoading(true);
    try {
        const status = await systemApi.status();
        systemStore.setSystemStatus(status);
    } catch (error) {
        console.error("Failed to get system status:", error);
    } finally {
        systemStore.setIsLoading(false);
    }
};

const formatSize = (size: number): string => {
    if (size < 1024) {
        return `${size} B`;
    } else if (size < 1024 * 1024) {
        return `${(size / 1024).toFixed(2)} KB`;
    } else if (size < 1024 * 1024 * 1024) {
        return `${(size / (1024 * 1024)).toFixed(2)} MB`;
    } else {
        return `${(size / (1024 * 1024 * 1024)).toFixed(2)} GB`;
    }
};

onMounted(() => {
    getSystemStatus();
});
</script>
