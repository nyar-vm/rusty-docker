<template>
  <div class="space-y-6">
    <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <h2 class="text-xl font-bold text-white">卷</h2>
      <div class="flex flex-col sm:flex-row gap-3">
        <button @click="createVolume" class="px-4 py-2 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all flex items-center justify-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
          </svg>
          创建卷
        </button>
        <button @click="listVolumes" class="px-4 py-2 bg-white/10 rounded-lg font-medium hover:bg-white/15 transition-all flex items-center justify-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          刷新
        </button>
      </div>
    </div>
    
    <!-- 创建卷卡片 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
      <h3 class="text-lg font-semibold text-white mb-4">创建卷</h3>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <input v-model="newVolumeName" placeholder="卷名称" class="px-4 py-2 bg-white/5 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed]">
        <select v-model="newVolumeDriver" class="px-4 py-2 bg-white/5 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed]">
          <option value="local">local</option>
        </select>
        <button @click="createVolume" class="px-4 py-2 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all flex items-center justify-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
          创建卷
        </button>
      </div>
    </div>
    
    <!-- 卷列表 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
      <h3 class="text-lg font-semibold text-white mb-4">卷列表</h3>
      
      <div v-if="volumes.length === 0" class="text-center py-10 text-white/60">
        暂无卷
      </div>
      
      <div v-else class="space-y-3">
        <div v-for="volume in volumes" :key="volume.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-4 bg-white/5 rounded-lg hover:bg-white/10 transition-all cursor-pointer" @click="showVolumeDetails(volume)">
          <div class="flex-1 mb-3 sm:mb-0">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 bg-amber-500/20 rounded-lg flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
                </svg>
              </div>
              <div>
                <h4 class="font-medium text-white">{{ volume.name }}</h4>
                <p class="text-sm text-white/60">{{ volume.mount_point }}</p>
              </div>
            </div>
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3">
            <span class="px-2 py-1 bg-white/10 text-white/80 rounded text-xs font-medium">
              {{ formatSize(volume.size) }}
            </span>
            <button @click.stop="deleteVolume(volume.id)" class="px-3 py-1 bg-red-500/20 border border-red-500/30 text-red-400 rounded hover:bg-red-500/30 transition-all text-sm">
              删除
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useVolumeStore } from "@/stores/modules/volumeStore";
import { volumeApi } from "@/api/docker";

const volumeStore = useVolumeStore();
const { volumes } = volumeStore;

const newVolumeName = ref("");
const newVolumeDriver = ref("local");

const listVolumes = async () => {
    volumeStore.setIsLoading(true);
    try {
        const data = await volumeApi.list();
        volumeStore.setVolumes(data);
    } catch (error) {
        console.error("Failed to list volumes:", error);
    } finally {
        volumeStore.setIsLoading(false);
    }
};

const createVolume = async () => {
    if (!newVolumeName.value) return;

    volumeStore.setIsLoading(true);
    try {
        const volume = await volumeApi.create({
            name: newVolumeName.value,
            driver: newVolumeDriver.value,
        });
        volumeStore.addVolume(volume);
        newVolumeName.value = "";
    } catch (error) {
        console.error("Failed to create volume:", error);
    } finally {
        volumeStore.setIsLoading(false);
    }
};

const deleteVolume = async (id: string) => {
    try {
        await volumeApi.remove(id);
        volumeStore.removeVolume(id);
    } catch (error) {
        console.error("Failed to delete volume:", error);
    }
};

const showVolumeDetails = (volume: any) => {
    volumeStore.setSelectedVolume(volume);
    // 这里可以通过路由或状态管理来切换到详情视图
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
    listVolumes();
});
</script>
