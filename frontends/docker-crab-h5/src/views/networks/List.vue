<template>
  <div class="space-y-6">
    <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <h2 class="text-xl font-bold text-white">网络</h2>
      <div class="flex flex-col sm:flex-row gap-3">
        <button @click="createNetwork" class="px-4 py-2 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all flex items-center justify-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
          创建网络
        </button>
        <button @click="listNetworks" class="px-4 py-2 bg-white/10 rounded-lg font-medium hover:bg-white/15 transition-all flex items-center justify-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          刷新
        </button>
      </div>
    </div>
    
    <!-- 创建网络卡片 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
      <h3 class="text-lg font-semibold text-white mb-4">创建网络</h3>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <input v-model="newNetworkName" placeholder="网络名称" class="px-4 py-2 bg-white/5 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed]">
        <select v-model="newNetworkDriver" class="px-4 py-2 bg-white/5 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed]">
          <option value="bridge">bridge</option>
          <option value="host">host</option>
          <option value="none">none</option>
        </select>
        <button @click="createNetwork" class="px-4 py-2 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all flex items-center justify-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
          创建网络
        </button>
      </div>
    </div>
    
    <!-- 网络列表 -->
    <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
      <h3 class="text-lg font-semibold text-white mb-4">网络列表</h3>
      
      <div v-if="networks.length === 0" class="text-center py-10 text-white/60">
        暂无网络
      </div>
      
      <div v-else class="space-y-3">
        <div v-for="network in networks" :key="network.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-4 bg-white/5 rounded-lg hover:bg-white/10 transition-all cursor-pointer" @click="showNetworkDetails(network)">
          <div class="flex-1 mb-3 sm:mb-0">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 bg-purple-500/20 rounded-lg flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              </div>
              <div>
                <h4 class="font-medium text-white">{{ network.name }}</h4>
                <p class="text-sm text-white/60">{{ network.driver }}</p>
              </div>
            </div>
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3">
            <span class="px-2 py-1 bg-white/10 text-white/80 rounded text-xs font-medium">
              {{ network.scope }}
            </span>
            <button @click.stop="deleteNetwork(network.id)" class="px-3 py-1 bg-red-500/20 border border-red-500/30 text-red-400 rounded hover:bg-red-500/30 transition-all text-sm">
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
import { useNetworkStore } from "@/stores/modules/networkStore";
import { networkApi } from "@/api/docker";

const networkStore = useNetworkStore();
const { networks } = networkStore;

const newNetworkName = ref("");
const newNetworkDriver = ref("bridge");

const listNetworks = async () => {
    networkStore.setIsLoading(true);
    try {
        const data = await networkApi.list();
        networkStore.setNetworks(data);
    } catch (error) {
        console.error("Failed to list networks:", error);
    } finally {
        networkStore.setIsLoading(false);
    }
};

const createNetwork = async () => {
    if (!newNetworkName.value) return;

    networkStore.setIsLoading(true);
    try {
        const network = await networkApi.create({
            name: newNetworkName.value,
            driver: newNetworkDriver.value,
        });
        networkStore.addNetwork(network);
        newNetworkName.value = "";
    } catch (error) {
        console.error("Failed to create network:", error);
    } finally {
        networkStore.setIsLoading(false);
    }
};

const deleteNetwork = async (id: string) => {
    try {
        await networkApi.remove(id);
        networkStore.removeNetwork(id);
    } catch (error) {
        console.error("Failed to delete network:", error);
    }
};

const showNetworkDetails = (network: any) => {
    networkStore.setSelectedNetwork(network);
    // 这里可以通过路由或状态管理来切换到详情视图
};

onMounted(() => {
    listNetworks();
});
</script>
