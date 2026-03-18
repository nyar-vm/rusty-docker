<template>
  <div class="space-y-6">
    <h2 class="text-xl font-semibold text-white">搜索结果</h2>
    
    <!-- 搜索统计 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
      <div class="flex flex-wrap gap-4">
        <div class="flex items-center gap-2">
          <span class="text-white/60 text-sm">搜索词:</span>
          <span class="text-white font-medium">{{ searchQuery || '无' }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-white/60 text-sm">标签:</span>
          <span class="text-white font-medium">{{ selectedTags.length > 0 ? selectedTags.join(', ') : '无' }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-white/60 text-sm">资源类型:</span>
          <span class="text-white font-medium">{{ resourceTypeLabel }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-white/60 text-sm">结果数:</span>
          <span class="text-white font-medium">{{ totalResults }}</span>
        </div>
      </div>
    </div>
    
    <!-- 容器结果 -->
    <div v-if="resourceType === 'all' || resourceType === 'containers'" class="space-y-3">
      <h3 class="text-lg font-semibold text-white">容器</h3>
      <div v-if="filteredContainers.length === 0" class="text-center py-6 text-white/40">
        暂无符合条件的容器
      </div>
      <div v-else class="space-y-2">
        <div v-for="container in filteredContainers" :key="container.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-all cursor-pointer">
          <div class="flex-1 mb-2 sm:mb-0">
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 bg-[#0db7ed]/20 rounded-lg flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-[#0db7ed]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                </svg>
              </div>
              <div>
                <h4 class="font-medium text-white text-sm">{{ container.name }}</h4>
                <p class="text-xs text-white/50">{{ container.image }}</p>
              </div>
            </div>
            <!-- 标签显示 -->
            <div v-if="getTagsByResourceId(container.id).length > 0" class="mt-2 flex flex-wrap gap-1">
              <span
                v-for="tag in getTagsByResourceId(container.id)"
                :key="tag"
                class="px-2 py-0.5 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs"
              >
                {{ tag }}
              </span>
            </div>
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-center gap-2">
            <span :class="container.status === 'Running' ? 'px-2 py-1 bg-green-600/10 text-green-400 rounded text-xs font-medium border border-green-600/20' : 'px-2 py-1 bg-gray-600/10 text-gray-400 rounded text-xs font-medium border border-gray-600/20'">
              {{ container.status }}
            </span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 镜像结果 -->
    <div v-if="resourceType === 'all' || resourceType === 'images'" class="space-y-3">
      <h3 class="text-lg font-semibold text-white">镜像</h3>
      <div v-if="filteredImages.length === 0" class="text-center py-6 text-white/40">
        暂无符合条件的镜像
      </div>
      <div v-else class="space-y-2">
        <div v-for="image in filteredImages" :key="image.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-all cursor-pointer">
          <div class="flex-1 mb-2 sm:mb-0">
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 bg-cyan-600/20 rounded-lg flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-cyan-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
                </svg>
              </div>
              <div>
                <h4 class="font-medium text-white text-sm">{{ image.repoTags?.[0] || '无标签' }}</h4>
                <p class="text-xs text-white/50">{{ image.id.substring(0, 12) }}</p>
              </div>
            </div>
            <!-- 标签显示 -->
            <div v-if="getTagsByResourceId(image.id).length > 0" class="mt-2 flex flex-wrap gap-1">
              <span
                v-for="tag in getTagsByResourceId(image.id)"
                :key="tag"
                class="px-2 py-0.5 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs"
              >
                {{ tag }}
              </span>
            </div>
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-center gap-2">
            <span class="px-2 py-1 bg-white/5 border border-white/10 text-white/70 rounded text-xs font-medium">
              {{ image.size }}
            </span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 网络结果 -->
    <div v-if="resourceType === 'all' || resourceType === 'networks'" class="space-y-3">
      <h3 class="text-lg font-semibold text-white">网络</h3>
      <div v-if="filteredNetworks.length === 0" class="text-center py-6 text-white/40">
        暂无符合条件的网络
      </div>
      <div v-else class="space-y-2">
        <div v-for="network in filteredNetworks" :key="network.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-all cursor-pointer">
          <div class="flex-1 mb-2 sm:mb-0">
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 bg-purple-500/20 rounded-lg flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              </div>
              <div>
                <h4 class="font-medium text-white text-sm">{{ network.name }}</h4>
                <p class="text-xs text-white/50">{{ network.driver }}</p>
              </div>
            </div>
            <!-- 标签显示 -->
            <div v-if="getTagsByResourceId(network.id).length > 0" class="mt-2 flex flex-wrap gap-1">
              <span
                v-for="tag in getTagsByResourceId(network.id)"
                :key="tag"
                class="px-2 py-0.5 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs"
              >
                {{ tag }}
              </span>
            </div>
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-center gap-2">
            <span class="px-2 py-1 bg-white/5 border border-white/10 text-white/70 rounded text-xs font-medium">
              {{ network.scope }}
            </span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 卷结果 -->
    <div v-if="resourceType === 'all' || resourceType === 'volumes'" class="space-y-3">
      <h3 class="text-lg font-semibold text-white">卷</h3>
      <div v-if="filteredVolumes.length === 0" class="text-center py-6 text-white/40">
        暂无符合条件的卷
      </div>
      <div v-else class="space-y-2">
        <div v-for="volume in filteredVolumes" :key="volume.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-all cursor-pointer">
          <div class="flex-1 mb-2 sm:mb-0">
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 bg-amber-500/20 rounded-lg flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
                </svg>
              </div>
              <div>
                <h4 class="font-medium text-white text-sm">{{ volume.name }}</h4>
                <p class="text-xs text-white/50">{{ volume.mount_point }}</p>
              </div>
            </div>
            <!-- 标签显示 -->
            <div v-if="getTagsByResourceId(volume.id).length > 0" class="mt-2 flex flex-wrap gap-1">
              <span
                v-for="tag in getTagsByResourceId(volume.id)"
                :key="tag"
                class="px-2 py-0.5 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs"
              >
                {{ tag }}
              </span>
            </div>
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-center gap-2">
            <span class="px-2 py-1 bg-white/5 border border-white/10 text-white/70 rounded text-xs font-medium">
              {{ formatSize(volume.size) }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useSearchStore } from "@/stores/modules/searchStore";
import { useTagStore } from "@/stores/modules/tagStore";
import { useContainerStore } from "@/stores/modules/containerStore";
import { useImageStore } from "@/stores/modules/imageStore";
import { useNetworkStore } from "@/stores/modules/networkStore";
import { useVolumeStore } from "@/stores/modules/volumeStore";

const searchStore = useSearchStore();
const tagStore = useTagStore();
const containerStore = useContainerStore();
const imageStore = useImageStore();
const networkStore = useNetworkStore();
const volumeStore = useVolumeStore();

const searchQuery = computed(() => searchStore.searchQuery);
const selectedTags = computed(() => searchStore.selectedTags);
const resourceType = computed(() => searchStore.resourceType);

const filteredContainers = computed(() => {
    return searchStore.filteredContainers(containerStore.containers);
});

const filteredImages = computed(() => {
    return searchStore.filteredImages(imageStore.images);
});

const filteredNetworks = computed(() => {
    return searchStore.filteredNetworks(networkStore.networks);
});

const filteredVolumes = computed(() => {
    return searchStore.filteredVolumes(volumeStore.volumes);
});

const totalResults = computed(() => {
    let count = 0;
    if (resourceType.value === "all" || resourceType.value === "containers") {
        count += filteredContainers.value.length;
    }
    if (resourceType.value === "all" || resourceType.value === "images") {
        count += filteredImages.value.length;
    }
    if (resourceType.value === "all" || resourceType.value === "networks") {
        count += filteredNetworks.value.length;
    }
    if (resourceType.value === "all" || resourceType.value === "volumes") {
        count += filteredVolumes.value.length;
    }
    return count;
});

const resourceTypeLabel = computed(() => {
    const labels = {
        all: "全部",
        containers: "容器",
        images: "镜像",
        networks: "网络",
        volumes: "卷",
    };
    return labels[resourceType.value];
});

const getTagsByResourceId = (resourceId: string) => {
    return tagStore.getTagsByResourceId(resourceId);
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
</script>
