<template>
  <div class="space-y-4">
    <!-- 搜索输入 -->
    <div class="relative">
      <input
        v-model="searchQuery"
        placeholder="搜索资源..."
        class="w-full px-4 py-2.5 pl-10 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] transition-all"
      />
      <svg xmlns="http://www.w3.org/2000/svg" class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-white/40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>
    
    <!-- 标签过滤 -->
    <div v-if="allTags.length > 0" class="space-y-2">
      <h4 class="text-sm font-medium text-white/70">按标签过滤</h4>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="tag in allTags"
          :key="tag"
          @click="toggleTag(tag)"
          :class="selectedTags.includes(tag) ? 'px-3 py-1 bg-[#0db7ed]/20 border border-[#0db7ed]/30 text-[#0db7ed] rounded-full text-xs' : 'px-3 py-1 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs hover:bg-white/10 transition-all'"
        >
          {{ tag }}
        </button>
      </div>
    </div>
    
    <!-- 资源类型选择 -->
    <div class="space-y-2">
      <h4 class="text-sm font-medium text-white/70">资源类型</h4>
      <div class="flex flex-wrap gap-2">
        <button
          @click="setResourceType('all')"
          :class="resourceType === 'all' ? 'px-3 py-1 bg-[#0db7ed]/20 border border-[#0db7ed]/30 text-[#0db7ed] rounded-full text-xs' : 'px-3 py-1 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs hover:bg-white/10 transition-all'"
        >
          全部
        </button>
        <button
          @click="setResourceType('containers')"
          :class="resourceType === 'containers' ? 'px-3 py-1 bg-[#0db7ed]/20 border border-[#0db7ed]/30 text-[#0db7ed] rounded-full text-xs' : 'px-3 py-1 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs hover:bg-white/10 transition-all'"
        >
          容器
        </button>
        <button
          @click="setResourceType('images')"
          :class="resourceType === 'images' ? 'px-3 py-1 bg-[#0db7ed]/20 border border-[#0db7ed]/30 text-[#0db7ed] rounded-full text-xs' : 'px-3 py-1 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs hover:bg-white/10 transition-all'"
        >
          镜像
        </button>
        <button
          @click="setResourceType('networks')"
          :class="resourceType === 'networks' ? 'px-3 py-1 bg-[#0db7ed]/20 border border-[#0db7ed]/30 text-[#0db7ed] rounded-full text-xs' : 'px-3 py-1 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs hover:bg-white/10 transition-all'"
        >
          网络
        </button>
        <button
          @click="setResourceType('volumes')"
          :class="resourceType === 'volumes' ? 'px-3 py-1 bg-[#0db7ed]/20 border border-[#0db7ed]/30 text-[#0db7ed] rounded-full text-xs' : 'px-3 py-1 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs hover:bg-white/10 transition-all'"
        >
          卷
        </button>
      </div>
    </div>
    
    <!-- 清除按钮 -->
    <button
      v-if="searchQuery || selectedTags.length > 0 || resourceType !== 'all'"
      @click="clearSearch"
      class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white/70 text-sm hover:bg-white/10 transition-all flex items-center justify-center gap-2"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      清除筛选
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useSearchStore } from "@/stores/modules/searchStore";
import { useTagStore } from "@/stores/modules/tagStore";

const searchStore = useSearchStore();
const tagStore = useTagStore();

const searchQuery = computed({
    get: () => searchStore.searchQuery,
    set: (value) => searchStore.setSearchQuery(value),
});

const selectedTags = computed({
    get: () => searchStore.selectedTags,
    set: (value) => searchStore.setSelectedTags(value),
});

const resourceType = computed({
    get: () => searchStore.resourceType,
    set: (value) => searchStore.setResourceType(value),
});

const allTags = computed(() => tagStore.getAllUniqueTags);

const toggleTag = (tag: string) => {
    searchStore.toggleTag(tag);
};

const setResourceType = (type: "all" | "containers" | "images" | "networks" | "volumes") => {
    searchStore.setResourceType(type);
};

const clearSearch = () => {
    searchStore.clearSearch();
};
</script>
