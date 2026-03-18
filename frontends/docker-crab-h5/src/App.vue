<template>
  <div class="w-full h-screen bg-[#121212] text-gray-100">
    <div v-if="!isAuthenticated" class="flex items-center justify-center w-full h-full p-6">
      <div class="w-full max-w-md bg-white/5 backdrop-blur-xl rounded-2xl border border-white/10 p-8 shadow-2xl">
        <div class="text-center mb-8">
          <div class="w-20 h-20 mx-auto mb-4 bg-gradient-to-br from-[#0db7ed] to-[#0071c5] rounded-xl shadow-lg shadow-[#0db7ed]/30 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <h1 class="text-3xl font-bold bg-gradient-to-r from-[#0db7ed] to-[#0071c5] bg-clip-text text-transparent mb-2">Docker Crab</h1>
          <p class="text-white/60 text-sm">专业的 Docker 管理工具</p>
        </div>
        <div class="space-y-3 mb-4">
          <input
            v-model="authToken"
            type="password"
            placeholder="输入网关令牌"
            class="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] focus:ring-1 focus:ring-[#0db7ed]/30 transition-all"
            @keyup.enter="authenticate"
          />
          <button
            @click="authenticate"
            :disabled="isLoading"
            class="w-full py-3 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-semibold text-sm cursor-pointer hover:translate-y-[-1px] hover:shadow-lg shadow-[#0db7ed]/20 transition-all disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {{ isLoading ? '连接中...' : '连接' }}
          </button>
        </div>
        <p v-if="authError" class="text-pink-400 text-center text-xs mb-4">{{ authError }}</p>
        <div class="text-center pt-4 border-t border-white/5">
          <p class="text-white/30 text-xs">Powered by OpenCrab</p>
        </div>
      </div>
    </div>

    <div v-else class="flex w-full h-full">
      <!-- 左侧导航栏 -->
      <aside class="w-64 bg-[#1e1e1e] border-r border-white/5 flex flex-col">
        <!-- 应用标题 -->
        <div class="flex items-center gap-3 px-6 py-4 border-b border-white/5">
          <div class="w-8 h-8 bg-gradient-to-br from-[#0db7ed] to-[#0071c5] rounded-lg shadow-lg shadow-[#0db7ed]/30 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <span class="text-base font-semibold text-white">Docker Crab</span>
        </div>
        
        <!-- 主导航标签 -->
        <nav class="flex-1 p-3 space-y-1">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            @click="switchTab(tab.id)"
            :class="currentTab === tab.id ? 'w-full px-3 py-2.5 bg-[#0db7ed]/20 text-[#0db7ed] rounded-lg font-medium flex items-center gap-2.5 transition-all' : 'w-full px-3 py-2.5 text-white/60 hover:text-white hover:bg-white/5 rounded-lg font-medium transition-all flex items-center gap-2.5'"
          >
            <svg v-if="tab.id === 'containers'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
            </svg>
            <svg v-else-if="tab.id === 'images'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
            </svg>
            <svg v-else-if="tab.id === 'templates'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
            </svg>
            <svg v-else-if="tab.id === 'networks'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
            <svg v-else-if="tab.id === 'volumes'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
            </svg>
            <svg v-else-if="tab.id === 'endpoints'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
            </svg>
            <svg v-else-if="tab.id === 'dockerhub'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <svg v-else-if="tab.id === 'system'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
            </svg>
            <svg v-else-if="tab.id === 'kubernetes'" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
            </svg>
            {{ tab.label }}
          </button>
        </nav>
        
        <!-- 系统状态 -->
        <div class="p-3 border-t border-white/5">
          <div class="flex items-center gap-2 px-3 py-2 bg-green-600/10 rounded-lg border border-green-600/20">
            <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse shadow-[0_0_0_3px_rgba(34,197,94,0.2)]"></div>
            <span class="text-green-400 text-xs font-semibold">Docker 运行中</span>
          </div>
          <button @click="restart" class="w-full mt-3 py-2 bg-white/5 rounded-lg cursor-pointer transition-all flex items-center justify-center gap-2 hover:bg-white/10">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-white/60 hover:text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span class="text-white/60 hover:text-white text-xs">重启</span>
          </button>
        </div>
      </aside>

      <!-- 主内容区域 -->
      <main class="flex-1 overflow-auto p-6 bg-[#1a1a1a]">
        <!-- 搜索标签内容 -->
        <div v-if="currentTab === 'search'" class="space-y-6">
          <h2 class="text-xl font-semibold text-white">高级搜索</h2>
          
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <!-- 搜索和过滤侧边栏 -->
            <div class="lg:col-span-1">
              <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4 sticky top-6">
                <h3 class="text-lg font-semibold text-white mb-4">搜索过滤</h3>
                <SearchFilter />
              </div>
            </div>
            
            <!-- 搜索结果 -->
            <div class="lg:col-span-2">
              <SearchResults />
            </div>
          </div>
        </div>
        
        <!-- 容器标签内容 -->
        <div v-else-if="currentTab === 'containers'" class="space-y-5">
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
          
          <!-- 容器列表 -->
          <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
            <h3 class="text-base font-semibold text-white mb-3">容器列表</h3>
            
            <div v-if="containers.length === 0" class="text-center py-8 text-white/40">
              暂无容器
            </div>
            
            <div v-else class="space-y-2">
              <div v-for="container in containers" :key="container.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-all cursor-pointer" @click="showContainerDetails(container)">
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

        <!-- 容器详情视图 -->
        <div v-if="currentTab === 'container-details'" class="space-y-5">
          <div class="flex items-center gap-3">
            <button @click="switchTab('containers')" class="flex items-center gap-2 px-4 py-2 bg-white/5 border border-white/10 rounded-lg hover:bg-white/10 transition-all text-sm">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              返回容器列表
            </button>
            <h2 class="text-xl font-semibold text-white">{{ selectedContainer?.name }} - 详情</h2>
          </div>
          
          <div v-if="selectedContainer" class="grid grid-cols-1 md:grid-cols-2 gap-5">
            <!-- 容器基本信息 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
              <h3 class="text-base font-semibold text-white mb-3">基本信息</h3>
              <div class="space-y-2">
                <div class="flex justify-between">
                  <span class="text-white/50 text-sm">容器 ID:</span>
                  <span class="text-white font-mono text-xs">{{ selectedContainer.id }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/50 text-sm">名称:</span>
                  <span class="text-white text-sm">{{ selectedContainer.name }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/50 text-sm">镜像:</span>
                  <span class="text-white text-sm">{{ selectedContainer.image }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/50 text-sm">状态:</span>
                  <span :class="selectedContainer.status === 'Running' ? 'text-green-400 text-sm' : 'text-gray-400 text-sm'">
                    {{ selectedContainer.status }}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/50 text-sm">创建时间:</span>
                  <span class="text-white text-xs">{{ selectedContainer.created }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/50 text-sm">端口映射:</span>
                  <span class="text-white text-xs">{{ selectedContainer.ports || '无' }}</span>
                </div>
              </div>
            </div>
            
            <!-- 容器操作 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
              <h3 class="text-base font-semibold text-white mb-3">操作</h3>
              <div class="grid grid-cols-2 gap-2">
                <button @click="startContainer(selectedContainer.id)" v-if="selectedContainer.status !== 'Running'" class="py-2.5 bg-green-600/10 border border-green-600/20 text-green-400 rounded hover:bg-green-600/20 transition-all font-medium text-sm">
                  启动容器
                </button>
                <button @click="stopContainer(selectedContainer.id)" v-if="selectedContainer.status === 'Running'" class="py-2.5 bg-yellow-600/10 border border-yellow-600/20 text-yellow-400 rounded hover:bg-yellow-600/20 transition-all font-medium text-sm">
                  停止容器
                </button>
                <button @click="restartContainer(selectedContainer.id)" class="py-2.5 bg-[#0db7ed]/10 border border-[#0db7ed]/20 text-[#0db7ed] rounded hover:bg-[#0db7ed]/20 transition-all font-medium text-sm">
                  重启容器
                </button>
                <button @click="deleteContainer(selectedContainer.id)" class="py-2.5 bg-red-600/10 border border-red-600/20 text-red-400 rounded hover:bg-red-600/20 transition-all font-medium text-sm">
                  删除容器
                </button>
              </div>
            </div>
            
            <!-- 标签管理 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
              <h3 class="text-base font-semibold text-white mb-3">标签管理</h3>
              <TagManager :resource-id="selectedContainer.id" />
            </div>
            
            <!-- 容器统计信息 -->
            <div class="md:col-span-2 bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
              <h3 class="text-base font-semibold text-white mb-3">统计信息</h3>
              <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div class="bg-white/5 rounded-lg p-3">
                  <div class="text-white/50 text-xs mb-1">CPU 使用率</div>
                  <div class="text-xl font-semibold text-white">{{ selectedContainer.cpuUsage || '0%' }}</div>
                </div>
                <div class="bg-white/5 rounded-lg p-3">
                  <div class="text-white/50 text-xs mb-1">内存使用</div>
                  <div class="text-xl font-semibold text-white">{{ selectedContainer.memoryUsage || '0 MB' }}</div>
                </div>
                <div class="bg-white/5 rounded-lg p-3">
                  <div class="text-white/50 text-xs mb-1">网络流量</div>
                  <div class="text-xl font-semibold text-white">{{ selectedContainer.networkUsage || '0 MB' }}</div>
                </div>
              </div>
            </div>
            
            <!-- 容器日志 -->
            <div class="md:col-span-2 bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
              <div class="flex items-center justify-between mb-3">
                <h3 class="text-base font-semibold text-white">容器日志</h3>
                <div class="flex items-center gap-2">
                  <input v-model="logSearchQuery" placeholder="搜索日志..." class="px-3 py-1.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] text-xs">
                  <button @click="fetchContainerLogs" class="px-3 py-1.5 bg-white/5 border border-white/10 rounded-lg hover:bg-white/10 transition-all flex items-center justify-center gap-1 text-xs">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    刷新
                  </button>
                </div>
              </div>
              <div class="bg-black/30 rounded-lg p-3 h-72 overflow-auto font-mono text-xs">
                <div v-if="isLoadingLogs" class="text-center py-4 text-white/40">加载中...</div>
                <div v-else-if="containerLogs.length === 0" class="text-center py-4 text-white/40">暂无日志</div>
                <div v-else class="space-y-1">
                  <div v-for="(log, index) in filteredLogs" :key="index" class="text-white/70" :class="{'bg-blue-600/10 p-1 rounded': log.includes(logSearchQuery) && logSearchQuery}">
                    {{ log }}
                  </div>
                </div>
              </div>
            </div>
            
            <!-- 容器终端 -->
            <div class="md:col-span-2 bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
              <div class="flex items-center justify-between mb-3">
                <h3 class="text-base font-semibold text-white">终端</h3>
                <button @click="executeTerminalCommand" class="px-3 py-1.5 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg hover:shadow-lg transition-all text-xs font-medium">
                  执行
                </button>
              </div>
              <div class="bg-black/30 rounded-lg p-3 h-72 overflow-auto font-mono text-xs mb-3">
                <div v-if="terminalOutput.length === 0" class="text-center py-4 text-white/40">在下方输入命令并点击执行</div>
                <div v-else class="space-y-1">
                  <div v-for="(line, index) in terminalOutput" :key="index" class="text-white/70">
                    {{ line }}
                  </div>
                </div>
              </div>
              <div class="flex gap-2">
                <input v-model="terminalCommand" placeholder="输入命令..." class="flex-1 px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] text-sm">
                <button @click="executeTerminalCommand" class="px-4 py-2 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg hover:shadow-lg transition-all font-medium text-sm">
                  执行
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- 镜像标签内容 -->
        <div v-else-if="currentTab === 'images'" class="space-y-5">
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
          
          <!-- 镜像列表 -->
          <div class="bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 p-4">
            <h3 class="text-base font-semibold text-white mb-3">镜像列表</h3>
            
            <div v-if="images.length === 0" class="text-center py-8 text-white/40">
              暂无镜像
            </div>
            
            <div v-else class="space-y-2">
              <div v-for="image in images" :key="image.id" class="flex flex-col sm:flex-row sm:items-center justify-between p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-all cursor-pointer" @click="showImageDetails(image)">
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
        
        <!-- 镜像详情视图 -->
        <div v-else-if="currentTab === 'image-details'" class="space-y-6">
          <div class="flex items-center gap-4">
            <button @click="switchTab('images')" class="flex items-center gap-2 px-4 py-2 bg-white/10 rounded-lg hover:bg-white/15 transition-all">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              返回镜像列表
            </button>
            <h2 class="text-xl font-bold text-white">{{ selectedImage?.repoTags?.[0] || '镜像详情' }}</h2>
          </div>
          
          <div v-if="selectedImage" class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- 镜像基本信息 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">基本信息</h3>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-white/60">镜像 ID:</span>
                  <span class="text-white font-mono text-sm">{{ selectedImage.id }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">标签:</span>
                  <span class="text-white">{{ selectedImage.repoTags?.join(', ') || '无' }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">大小:</span>
                  <span class="text-white">{{ selectedImage.size }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">创建时间:</span>
                  <span class="text-white text-sm">{{ selectedImage.created }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">架构:</span>
                  <span class="text-white">{{ selectedImage.architecture || '未知' }}</span>
                </div>
              </div>
            </div>
            
            <!-- 镜像操作 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">操作</h3>
              <div class="grid grid-cols-1 gap-3">
                <button @click="deleteImage(selectedImage.id)" class="py-3 bg-red-500/20 border border-red-500/30 text-red-400 rounded hover:bg-red-500/30 transition-all font-medium">
                  删除镜像
                </button>
                <button class="py-3 bg-blue-500/20 border border-blue-500/30 text-blue-400 rounded hover:bg-blue-500/30 transition-all font-medium">
                  从镜像创建容器
                </button>
              </div>
            </div>
            
            <!-- 标签管理 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">标签管理</h3>
              <TagManager :resource-id="selectedImage.id" />
            </div>
            
            <!-- 镜像历史 -->
            <div class="md:col-span-2 bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">镜像历史</h3>
              <div class="space-y-3">
                <div v-for="(history, index) in selectedImage.history || []" :key="index" class="p-3 bg-white/5 rounded-lg">
                  <div class="text-white font-medium mb-1">{{ history.created || '未知时间' }}</div>
                  <div class="text-white/60 text-sm">{{ history.createdBy || '未知操作' }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 网络标签内容 -->
        <div v-else-if="currentTab === 'networks'" class="space-y-6">
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

        <!-- 网络详情视图 -->
        <div v-else-if="currentTab === 'network-details'" class="space-y-6">
          <div class="flex items-center gap-4">
            <button @click="switchTab('networks')" class="flex items-center gap-2 px-4 py-2 bg-white/10 rounded-lg hover:bg-white/15 transition-all">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              返回网络列表
            </button>
            <h2 class="text-xl font-bold text-white">{{ selectedNetwork?.name }} - 详情</h2>
          </div>
          
          <div v-if="selectedNetwork" class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- 网络基本信息 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">基本信息</h3>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-white/60">网络 ID:</span>
                  <span class="text-white font-mono text-sm">{{ selectedNetwork.id }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">名称:</span>
                  <span class="text-white">{{ selectedNetwork.name }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">驱动:</span>
                  <span class="text-white">{{ selectedNetwork.driver }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">作用域:</span>
                  <span class="text-white">{{ selectedNetwork.scope }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">启用 IPv6:</span>
                  <span class="text-white">{{ selectedNetwork.enable_ipv6 ? '是' : '否' }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">内部网络:</span>
                  <span class="text-white">{{ selectedNetwork.internal ? '是' : '否' }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">可附加:</span>
                  <span class="text-white">{{ selectedNetwork.attachable ? '是' : '否' }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">入口网络:</span>
                  <span class="text-white">{{ selectedNetwork.ingress ? '是' : '否' }}</span>
                </div>
              </div>
            </div>
            
            <!-- 网络操作 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">操作</h3>
              <div class="grid grid-cols-1 gap-3">
                <button @click="deleteNetwork(selectedNetwork.id)" class="py-3 bg-red-500/20 border border-red-500/30 text-red-400 rounded hover:bg-red-500/30 transition-all font-medium">
                  删除网络
                </button>
              </div>
            </div>
            
            <!-- 标签管理 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">标签管理</h3>
              <TagManager :resource-id="selectedNetwork.id" />
            </div>
            
            <!-- 容器信息 -->
            <div class="md:col-span-2 bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">连接的容器</h3>
              
              <div v-if="Object.keys(selectedNetwork.containers || {}).length === 0" class="text-center py-6 text-white/60">
                暂无容器连接到该网络
              </div>
              
              <div v-else class="space-y-3">
                <div v-for="(container, key) in selectedNetwork.containers" :key="key" class="p-4 bg-white/5 rounded-lg">
                  <div class="flex items-center gap-3">
                    <div class="w-8 h-8 bg-blue-500/20 rounded-lg flex items-center justify-center">
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                      </svg>
                    </div>
                    <div class="flex-1">
                      <h4 class="font-medium text-white">{{ container.name }}</h4>
                      <p class="text-sm text-white/60">MAC: {{ container.mac_address }} | IPv4: {{ container.ipv4_address }}</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 卷标签内容 -->
        <div v-else-if="currentTab === 'volumes'" class="space-y-6">
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
        
        <!-- 卷详情视图 -->
        <div v-else-if="currentTab === 'volume-details'" class="space-y-6">
          <div class="flex items-center gap-4">
            <button @click="switchTab('volumes')" class="flex items-center gap-2 px-4 py-2 bg-white/10 rounded-lg hover:bg-white/15 transition-all">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              返回卷列表
            </button>
            <h2 class="text-xl font-bold text-white">{{ selectedVolume?.name }} - 详情</h2>
          </div>
          
          <div v-if="selectedVolume" class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- 卷基本信息 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">基本信息</h3>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-white/60">卷 ID:</span>
                  <span class="text-white font-mono text-sm">{{ selectedVolume.id }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">名称:</span>
                  <span class="text-white">{{ selectedVolume.name }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">大小:</span>
                  <span class="text-white">{{ formatSize(selectedVolume.size) }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">创建时间:</span>
                  <span class="text-white text-sm">{{ formatDate(selectedVolume.created_at) }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">挂载点:</span>
                  <span class="text-white text-sm">{{ selectedVolume.mount_point }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">驱动:</span>
                  <span class="text-white">{{ selectedVolume.driver }}</span>
                </div>
              </div>
            </div>
            
            <!-- 卷操作 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">操作</h3>
              <div class="grid grid-cols-1 gap-3">
                <button @click="deleteVolume(selectedVolume.id)" class="py-3 bg-red-500/20 border border-red-500/30 text-red-400 rounded hover:bg-red-500/30 transition-all font-medium">
                  删除卷
                </button>
              </div>
            </div>
            
            <!-- 标签管理 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">标签管理</h3>
              <TagManager :resource-id="selectedVolume.id" />
            </div>
            
            <!-- 卷使用情况 -->
            <div class="md:col-span-2 bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">使用情况</h3>
              
              <div v-if="selectedVolume.used_by.length === 0" class="text-center py-6 text-white/60">
                该卷未被任何容器使用
              </div>
              
              <div v-else class="space-y-3">
                <div v-for="(container, index) in selectedVolume.used_by" :key="index" class="p-4 bg-white/5 rounded-lg">
                  <div class="flex items-center gap-3">
                    <div class="w-8 h-8 bg-blue-500/20 rounded-lg flex items-center justify-center">
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                      </svg>
                    </div>
                    <div class="flex-1">
                      <h4 class="font-medium text-white">{{ container }}</h4>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 端点管理标签内容 -->
        <div v-else-if="currentTab === 'endpoints'" class="space-y-6">
          <EndpointsList />
        </div>

        <!-- 系统状态标签内容 -->
        <div v-else-if="currentTab === 'system'" class="space-y-6">
          <div class="flex items-center gap-4">
            <h2 class="text-xl font-bold text-white">系统状态</h2>
            <button @click="getSystemStatus" class="px-4 py-2 bg-white/10 rounded-lg font-medium hover:bg-white/15 transition-all flex items-center justify-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              刷新
            </button>
          </div>
          
          <!-- 系统状态卡片 -->
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <!-- 容器状态 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">容器状态</h3>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-white/60">运行中:</span>
                  <span class="text-green-400 font-semibold">{{ systemStatus?.container_stats?.running || 0 }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">已停止:</span>
                  <span class="text-gray-400 font-semibold">{{ systemStatus?.container_stats?.stopped || 0 }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">总数:</span>
                  <span class="text-white font-semibold">{{ systemStatus?.container_stats?.total || 0 }}</span>
                </div>
              </div>
            </div>
            
            <!-- 系统资源 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">系统资源</h3>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-white/60">CPU 使用率:</span>
                  <span class="text-white font-semibold">{{ systemStatus?.resource_usage?.cpu_usage || 0 }}%</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">内存使用:</span>
                  <span class="text-white font-semibold">{{ (systemStatus?.resource_usage?.memory_used || 0) }} MB / {{ (systemStatus?.resource_usage?.memory_total || 0) }} MB</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">存储使用:</span>
                  <span class="text-white font-semibold">{{ (systemStatus?.resource_usage?.storage_used || 0) }} GB / {{ (systemStatus?.resource_usage?.storage_total || 0) }} GB</span>
                </div>
              </div>
            </div>
            
            <!-- 系统信息 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">系统信息</h3>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-white/60">操作系统:</span>
                  <span class="text-white font-semibold">{{ systemStatus?.system_info?.os_type }} {{ systemStatus?.system_info?.os_version }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">架构:</span>
                  <span class="text-white font-semibold">{{ systemStatus?.system_info?.architecture }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">CPU 核心:</span>
                  <span class="text-white font-semibold">{{ systemStatus?.system_info?.cpu_cores }}</span>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Docker 状态 -->
          <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
            <h3 class="text-lg font-semibold text-white mb-4">Docker 状态</h3>
            <div class="flex items-center gap-3">
              <div :class="systemStatus?.daemon_status === 'Running' ? 'w-3 h-3 bg-green-500 rounded-full animate-pulse' : 'w-3 h-3 bg-red-500 rounded-full'">
              </div>
              <span :class="systemStatus?.daemon_status === 'Running' ? 'text-green-400 font-medium' : 'text-red-400 font-medium'">
                {{ systemStatus?.daemon_status || 'Unknown' }}
              </span>
            </div>
          </div>
        </div>

        <!-- 监控标签内容 -->
        <div v-else-if="currentTab === 'monitor'">
          <Monitor />
        </div>
        
        <!-- 模板管理标签内容 -->
        <div v-else-if="currentTab === 'templates'" class="space-y-6">
          <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
            <h2 class="text-xl font-bold text-white">模板管理</h2>
            <div class="flex flex-col sm:flex-row gap-3">
              <input v-model="templateSearchQuery" placeholder="搜索模板..." class="px-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed]">
              <select v-model="templateCategoryFilter" class="px-4 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed]">
                <option value="all">全部分类</option>
                <option value="Web 服务">Web 服务</option>
                <option value="数据库">数据库</option>
                <option value="开发工具">开发工具</option>
              </select>
            </div>
          </div>
          
          <!-- 模板列表 -->
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            <div v-for="template in getFilteredTemplates()" :key="template.id" class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5 hover:bg-white/10 transition-all cursor-pointer" @click="showTemplateDetails(template)">
              <div class="flex items-center gap-4 mb-4">
                <div class="w-12 h-12 rounded-lg overflow-hidden flex items-center justify-center bg-white/10">
                  <img :src="template.icon" alt="{{ template.name }}" class="w-8 h-8 object-contain">
                </div>
                <div>
                  <h3 class="text-lg font-semibold text-white">{{ template.name }}</h3>
                  <span class="px-2 py-1 bg-white/10 text-white/80 rounded text-xs font-medium">{{ template.category }}</span>
                </div>
              </div>
              <p class="text-white/60 text-sm mb-4">{{ template.description }}</p>
              <div class="space-y-2 mb-4">
                <div class="flex items-center gap-2 text-sm text-white/70">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
                  </svg>
                  {{ template.image }}
                </div>
                <div v-if="template.ports.length > 0" class="flex items-center gap-2 text-sm text-white/70">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                  </svg>
                  {{ template.ports.join(', ') }}
                </div>
              </div>
              <button @click.stop="showTemplateDetails(template)" class="w-full py-2 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all text-sm">
                查看详情
              </button>
            </div>
          </div>
        </div>
        
        <!-- 模板详情视图 -->
        <div v-else-if="currentTab === 'template-details'" class="space-y-6">
          <div class="flex items-center gap-4">
            <button @click="switchTab('templates')" class="flex items-center gap-2 px-4 py-2 bg-white/10 rounded-lg hover:bg-white/15 transition-all">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              返回模板列表
            </button>
            <h2 class="text-xl font-bold text-white">{{ selectedTemplate?.name }} - 详情</h2>
          </div>
          
          <div v-if="selectedTemplate" class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- 模板基本信息 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <div class="flex items-center gap-4 mb-4">
                <div class="w-16 h-16 rounded-lg overflow-hidden flex items-center justify-center bg-white/10">
                  <img :src="selectedTemplate.icon" alt="{{ selectedTemplate.name }}" class="w-12 h-12 object-contain">
                </div>
                <div>
                  <h3 class="text-xl font-semibold text-white">{{ selectedTemplate.name }}</h3>
                  <span class="px-2 py-1 bg-white/10 text-white/80 rounded text-xs font-medium">{{ selectedTemplate.category }}</span>
                </div>
              </div>
              <p class="text-white/60 mb-4">{{ selectedTemplate.description }}</p>
              <div class="space-y-3">
                <div class="flex justify-between">
                  <span class="text-white/60">镜像:</span>
                  <span class="text-white">{{ selectedTemplate.image }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">端口映射:</span>
                  <span class="text-white">{{ selectedTemplate.ports.join(', ') || '无' }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">卷映射:</span>
                  <span class="text-white">{{ selectedTemplate.volumes.join(', ') || '无' }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-white/60">环境变量:</span>
                  <span class="text-white">{{ selectedTemplate.env.join(', ') || '无' }}</span>
                </div>
              </div>
            </div>
            
            <!-- 部署模板 -->
            <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
              <h3 class="text-lg font-semibold text-white mb-4">部署模板</h3>
              <div class="space-y-4">
                <div>
                  <label class="block text-white/60 text-sm mb-2">容器名称</label>
                  <input v-model="newContainerName" placeholder="输入容器名称" class="w-full px-4 py-2 bg-white/5 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed]">
                </div>
                <button @click="deployTemplate(selectedTemplate, newContainerName)" :disabled="isDeployingTemplate" class="w-full py-3 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all">
                  {{ isDeployingTemplate ? '部署中...' : '部署模板' }}
                </button>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Kubernetes 标签内容 -->
        <div v-else-if="currentTab === 'kubernetes'" class="space-y-6">
          <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
            <h2 class="text-xl font-bold text-white">Kubernetes</h2>
            <div class="flex flex-col sm:flex-row gap-3">
              <button @click="listK8sPods" class="px-4 py-2 bg-gradient-to-r from-[#0db7ed] to-[#0071c5] rounded-lg font-medium hover:shadow-lg transition-all flex items-center justify-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                </svg>
                查看 Pods
              </button>
              <button @click="listK8sServices" class="px-4 py-2 bg-white/10 rounded-lg font-medium hover:bg-white/15 transition-all flex items-center justify-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
                </svg>
                查看服务
              </button>
            </div>
          </div>
          
          <!-- Kubernetes 集群状态 -->
          <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
            <h3 class="text-lg font-semibold text-white mb-4">集群状态</h3>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div class="bg-white/5 rounded-lg p-3">
                <div class="text-white/60 text-sm mb-1">节点数量</div>
                <div class="text-xl font-semibold text-white">{{ k8sStatus?.nodes || 1 }}</div>
              </div>
              <div class="bg-white/5 rounded-lg p-3">
                <div class="text-white/60 text-sm mb-1">运行中 Pods</div>
                <div class="text-xl font-semibold text-white">{{ k8sStatus?.runningPods || 2 }}</div>
              </div>
              <div class="bg-white/5 rounded-lg p-3">
                <div class="text-white/60 text-sm mb-1">服务数量</div>
                <div class="text-xl font-semibold text-white">{{ k8sStatus?.services || 1 }}</div>
              </div>
            </div>
          </div>
          
          <!-- Kubernetes Pods 列表 -->
          <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
            <h3 class="text-lg font-semibold text-white mb-4">Pods</h3>
            
            <div v-if="k8sPods.length === 0" class="text-center py-10 text-white/60">
              暂无 Pods
            </div>
            
            <div v-else class="space-y-3">
              <div v-for="pod in k8sPods" :key="pod.metadata.name" class="flex flex-col sm:flex-row sm:items-center justify-between p-4 bg-white/5 rounded-lg hover:bg-white/10 transition-all">
                <div class="flex-1 mb-3 sm:mb-0">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 bg-blue-500/20 rounded-lg flex items-center justify-center">
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-medium text-white">{{ pod.metadata.name }}</h4>
                      <p class="text-sm text-white/60">{{ pod.metadata.namespace || 'default' }}</p>
                    </div>
                  </div>
                </div>
                <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3">
                  <span :class="pod.status.phase === 'Running' ? 'px-2 py-1 bg-green-500/20 text-green-400 rounded text-xs font-medium border border-green-500/30' : 'px-2 py-1 bg-gray-500/20 text-gray-400 rounded text-xs font-medium border border-gray-500/30'">
                    {{ pod.status.phase || 'Unknown' }}
                  </span>
                  <button @click="describeK8sPod(pod.metadata.name)" class="px-3 py-1 bg-white/10 border border-white/20 text-white/80 rounded hover:bg-white/20 transition-all text-sm">
                    详情
                  </button>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Kubernetes 服务列表 -->
          <div class="bg-white/5 backdrop-blur-xl rounded-xl border border-white/10 p-5">
            <h3 class="text-lg font-semibold text-white mb-4">服务</h3>
            
            <div v-if="k8sServices.length === 0" class="text-center py-10 text-white/60">
              暂无服务
            </div>
            
            <div v-else class="space-y-3">
              <div v-for="service in k8sServices" :key="service.metadata.name" class="flex flex-col sm:flex-row sm:items-center justify-between p-4 bg-white/5 rounded-lg hover:bg-white/10 transition-all">
                <div class="flex-1 mb-3 sm:mb-0">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 bg-purple-500/20 rounded-lg flex items-center justify-center">
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                      </svg>
                    </div>
                    <div>
                      <h4 class="font-medium text-white">{{ service.metadata.name }}</h4>
                      <p class="text-sm text-white/60">{{ service.spec.type || 'ClusterIP' }}</p>
                    </div>
                  </div>
                </div>
                <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3">
                  <span class="px-2 py-1 bg-white/10 text-white/80 rounded text-xs font-medium">
                    {{ service.spec.ports?.[0]?.port || 0 }}
                  </span>
                  <button @click="describeK8sService(service.metadata.name)" class="px-3 py-1 bg-white/10 border border-white/20 text-white/80 rounded hover:bg-white/20 transition-all text-sm">
                    详情
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import EndpointsList from "./views/endpoints/List.vue";
import Monitor from "./views/system/Monitor.vue";
import SearchFilter from "./components/SearchFilter.vue";
import SearchResults from "./components/SearchResults.vue";
import TagManager from "./components/TagManager.vue";

const isAuthenticated = ref(false);
const isLoading = ref(false);
const authToken = ref("");
const authError = ref("");
const currentTab = ref("containers");

// 容器管理相关变量
const containers = ref<any[]>([]);
const newContainerImage = ref("alpine");
const newContainerName = ref("");
const newContainerPorts = ref("");
const selectedContainer = ref<any>(null);
let statusUpdateInterval: number | undefined;

// 容器日志相关变量
const containerLogs = ref<string[]>([]);
const isLoadingLogs = ref(false);
const logSearchQuery = ref("");

// 容器终端相关变量
const terminalCommand = ref("");
const terminalOutput = ref<string[]>([]);

// 镜像管理相关变量
const images = ref<any[]>([]);
const newImageName = ref("");
const isPullingImage = ref(false);
const selectedImage = ref<any>(null);
const isManagingImage = ref(false);

// 网络管理相关变量
const networks = ref<any[]>([]);
const newNetworkName = ref("");
const newNetworkDriver = ref("bridge");
const selectedNetwork = ref<any>(null);
const isManagingNetwork = ref(false);

// 卷管理相关变量
const volumes = ref<any[]>([]);
const newVolumeName = ref("");
const newVolumeDriver = ref("local");
const selectedVolume = ref<any>(null);
const isManagingVolume = ref(false);

// 系统状态相关变量
const systemStatus = ref<any>(null);

// Docker Hub 相关变量
// const dockerHubSearchQuery = ref('')
// const dockerHubSearchResults = ref<any[]>([])
// const isSearchingDockerHub = ref(false)
// const selectedDockerHubImage = ref<any>(null)

// 模板管理相关变量
const templates = ref<any[]>([
    {
        id: "1",
        name: "Nginx",
        description: "高性能 Web 服务器",
        image: "nginx:latest",
        category: "Web 服务",
        ports: ["80:80"],
        volumes: [],
        env: [],
        icon: "https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=nginx%20logo%20simple%20icon&image_size=square",
    },
    {
        id: "2",
        name: "MySQL",
        description: "关系型数据库",
        image: "mysql:latest",
        category: "数据库",
        ports: ["3306:3306"],
        volumes: ["mysql-data:/var/lib/mysql"],
        env: ["MYSQL_ROOT_PASSWORD=root", "MYSQL_DATABASE=app"],
        icon: "https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=mysql%20logo%20simple%20icon&image_size=square",
    },
    {
        id: "3",
        name: "Redis",
        description: "内存数据库",
        image: "redis:latest",
        category: "数据库",
        ports: ["6379:6379"],
        volumes: ["redis-data:/data"],
        env: [],
        icon: "https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=redis%20logo%20simple%20icon&image_size=square",
    },
    {
        id: "4",
        name: "Node.js",
        description: "JavaScript 运行时",
        image: "node:latest",
        category: "开发工具",
        ports: ["3000:3000"],
        volumes: ["node-app:/app"],
        env: [],
        icon: "https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=node.js%20logo%20simple%20icon&image_size=square",
    },
    {
        id: "5",
        name: "PostgreSQL",
        description: "开源关系型数据库",
        image: "postgres:latest",
        category: "数据库",
        ports: ["5432:5432"],
        volumes: ["postgres-data:/var/lib/postgresql/data"],
        env: ["POSTGRES_PASSWORD=postgres", "POSTGRES_DB=app"],
        icon: "https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=postgresql%20logo%20simple%20icon&image_size=square",
    },
    {
        id: "6",
        name: "MongoDB",
        description: "NoSQL 数据库",
        image: "mongo:latest",
        category: "数据库",
        ports: ["27017:27017"],
        volumes: ["mongo-data:/data/db"],
        env: ["MONGO_INITDB_ROOT_USERNAME=root", "MONGO_INITDB_ROOT_PASSWORD=root"],
        icon: "https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=mongodb%20logo%20simple%20icon&image_size=square",
    },
]);
const selectedTemplate = ref<any>(null);
const templateSearchQuery = ref("");
const templateCategoryFilter = ref("all");
const isDeployingTemplate = ref(false);

// Kubernetes 相关变量
const k8sStatus = ref<any>({
    nodes: 1,
    runningPods: 2,
    services: 1,
});
const k8sPods = ref<any[]>([
    {
        metadata: {
            name: "nginx-12345",
            namespace: "default",
        },
        status: {
            phase: "Running",
        },
    },
    {
        metadata: {
            name: "redis-67890",
            namespace: "default",
        },
        status: {
            phase: "Running",
        },
    },
]);
const k8sServices = ref<any[]>([
    {
        metadata: {
            name: "nginx-service",
        },
        spec: {
            type: "ClusterIP",
            ports: [
                {
                    port: 80,
                },
            ],
        },
    },
]);

// Docker Desktop 风格的导航标签
const tabs = [
    { id: "search", label: "搜索" },
    { id: "containers", label: "容器" },
    { id: "images", label: "镜像" },
    { id: "templates", label: "模板" },
    { id: "dockerhub", label: "Docker Hub" },
    { id: "networks", label: "网络" },
    { id: "volumes", label: "卷" },
    { id: "endpoints", label: "端点" },
    { id: "kubernetes", label: "Kubernetes" },
    { id: "system", label: "系统" },
];

function authenticate() {
    if (!authToken.value.trim()) {
        authError.value = "请输入令牌";
        return;
    }

    isLoading.value = true;
    authError.value = "";

    setTimeout(() => {
        isAuthenticated.value = true;
        isLoading.value = false;
        ElMessage.success("欢迎使用 Docker Crab！");
    }, 800);
}

function switchTab(tabId: string) {
    currentTab.value = tabId;
    if (tabId !== "container-details") {
        selectedContainer.value = null;
    }
    if (tabId !== "image-details") {
        selectedImage.value = null;
    }
    if (tabId !== "network-details") {
        selectedNetwork.value = null;
    }
    if (tabId !== "volume-details") {
        selectedVolume.value = null;
    }
    // if (tabId !== 'dockerhub-details') {
    //   selectedDockerHubImage.value = null
    // }
    if (tabId !== "template-details") {
        selectedTemplate.value = null;
    }
}

function restart() {
    ElMessage.info("正在重启...");
}

// 模板管理相关方法
function getFilteredTemplates() {
    let filtered = templates.value;

    // 按分类过滤
    if (templateCategoryFilter.value !== "all") {
        filtered = filtered.filter(
            (template) => template.category === templateCategoryFilter.value,
        );
    }

    // 按搜索词过滤
    if (templateSearchQuery.value) {
        const query = templateSearchQuery.value.toLowerCase();
        filtered = filtered.filter(
            (template) =>
                template.name.toLowerCase().includes(query) ||
                template.description.toLowerCase().includes(query) ||
                template.category.toLowerCase().includes(query),
        );
    }

    return filtered;
}

function showTemplateDetails(template: any) {
    selectedTemplate.value = template;
    currentTab.value = "template-details";
}

async function deployTemplate(template: any, containerName: string) {
    if (!containerName) {
        ElMessage.warning("请输入容器名称");
        return;
    }

    isDeployingTemplate.value = true;

    try {
        await invoke("run_container", {
            image: template.image,
            name: containerName,
            ports: template.ports,
            volumes: template.volumes,
            env: template.env,
        });

        ElMessage.success(`模板 ${template.name} 部署成功！`);
        await listContainers();
    } catch (error) {
        ElMessage.error(`部署模板失败: ${error}`);
    } finally {
        isDeployingTemplate.value = false;
    }
}

// Docker Hub 相关方法
// async function searchDockerHub() {
//   if (!dockerHubSearchQuery.value) {
//     ElMessage.warning('请输入搜索关键词')
//     return
//   }
//
//   isSearchingDockerHub.value = true
//
//   try {
//     // 模拟 Docker Hub 搜索结果
//     const mockResults = [
//       {
//         name: 'nginx',
//         description: 'Official build of Nginx.',
//         stars: 18000,
//         pulls: 5000000000,
//         official: true,
//         tags: ['latest', '1.25', '1.24', '1.23']
//       },
//       {
//         name: 'alpine',
//         description: 'A minimal Docker image based on Alpine Linux with a complete package index and only 5 MB in size!',
//         stars: 15000,
//         pulls: 4000000000,
//         official: true,
//         tags: ['latest', '3.18', '3.17', '3.16']
//       },
//       {
//         name: 'ubuntu',
//         description: 'Ubuntu is a Debian-based Linux operating system based on free software.',
//         stars: 14000,
//         pulls: 3000000000,
//         official: true,
//         tags: ['latest', '22.04', '20.04', '18.04']
//       },
//       {
//         name: 'node',
//         description: 'Node.js is a JavaScript-based platform for server-side and networking applications.',
//         stars: 13000,
//         pulls: 2500000000,
//         official: true,
//         tags: ['latest', '18', '16', '14']
//       },
//       {
//         name: 'mysql',
//         description: 'MySQL is a widely used, open-source relational database management system (RDBMS).',
//         stars: 12000,
//         pulls: 2000000000,
//         official: true,
//         tags: ['latest', '8.0', '5.7', '5.6']
//       }
//     ]
//
//     // 过滤搜索结果
//     dockerHubSearchResults.value = mockResults.filter(image =>
//       image.name.toLowerCase().includes(dockerHubSearchQuery.value.toLowerCase())
//     )
//   } catch (error) {
//     ElMessage.error(`搜索 Docker Hub 失败: ${error}`)
//   } finally {
//     isSearchingDockerHub.value = false
//   }
// }

// function showDockerHubImageDetails(image: any) {
//   selectedDockerHubImage.value = image
//   currentTab.value = 'dockerhub-details'
// }

// async function pullDockerHubImage(image: any, tag: string) {
//   isPullingImage.value = true
//
//   try {
//     await invoke('pull_image', { imageName: `${image.name}:${tag}` })
//     ElMessage.success(`镜像 ${image.name}:${tag} 拉取成功！`)
//     await listImages()
//   } catch (error) {
//     ElMessage.error(`拉取镜像失败: ${error}`)
//   } finally {
//     isPullingImage.value = false
//   }
// }

// 容器管理相关方法
async function listContainers() {
    try {
        const result = await invoke("list_containers", { all: true });
        containers.value = result as any[];

        // 如果当前查看的容器在列表中，更新其状态
        if (selectedContainer.value) {
            const updatedContainer = containers.value.find(
                (c) => c.id === selectedContainer.value.id,
            );
            if (updatedContainer) {
                selectedContainer.value = updatedContainer;
            }
        }
    } catch (error) {
        ElMessage.error(`获取容器列表失败: ${error}`);
    }
}

const isRunningContainer = ref(false);

async function runContainer() {
    if (!newContainerImage.value) {
        ElMessage.warning("请输入镜像名称");
        return;
    }

    isRunningContainer.value = true;

    try {
        const ports = newContainerPorts.value
            ? newContainerPorts.value.split(",").map((p) => p.trim())
            : [];

        await invoke("run_container", {
            image: newContainerImage.value,
            name: newContainerName.value || undefined,
            ports,
        });

        ElMessage.success("容器创建成功！");
        await listContainers();

        // 重置表单
        newContainerName.value = "";
        newContainerPorts.value = "";
    } catch (error) {
        ElMessage.error(`创建容器失败: ${error}`);
    } finally {
        isRunningContainer.value = false;
    }
}

// 容器操作加载状态
const isOperatingContainer = ref(false);

async function startContainer(containerId: string) {
    isOperatingContainer.value = true;
    try {
        await invoke("start_container", { containerId });
        ElMessage.success("容器启动成功！");
        await listContainers();
    } catch (error) {
        ElMessage.error(`启动容器失败: ${error}`);
    } finally {
        isOperatingContainer.value = false;
    }
}

async function stopContainer(containerId: string) {
    isOperatingContainer.value = true;
    try {
        await invoke("stop_container", { containerId });
        ElMessage.success("容器停止成功！");
        await listContainers();
    } catch (error) {
        ElMessage.error(`停止容器失败: ${error}`);
    } finally {
        isOperatingContainer.value = false;
    }
}

async function restartContainer(containerId: string) {
    isOperatingContainer.value = true;
    try {
        await invoke("restart_container", { containerId });
        ElMessage.success("容器重启成功！");
        await listContainers();
    } catch (error) {
        ElMessage.error(`重启容器失败: ${error}`);
    } finally {
        isOperatingContainer.value = false;
    }
}

async function deleteContainer(containerId: string) {
    isOperatingContainer.value = true;
    try {
        await invoke("delete_container", { containerId });
        ElMessage.success("容器删除成功！");
        await listContainers();

        // 如果删除的是当前查看的容器，返回容器列表
        if (selectedContainer.value && selectedContainer.value.id === containerId) {
            switchTab("containers");
        }
    } catch (error) {
        ElMessage.error(`删除容器失败: ${error}`);
    } finally {
        isOperatingContainer.value = false;
    }
}

function showContainerDetails(container: any) {
    selectedContainer.value = container;
    currentTab.value = "container-details";
    // 自动获取容器日志
    fetchContainerLogs();
}

// 容器日志相关方法
async function fetchContainerLogs() {
    if (!selectedContainer.value) return;

    isLoadingLogs.value = true;
    try {
        const result = await invoke("get_container_logs", {
            containerId: selectedContainer.value.id,
            lines: 100,
            follow: false,
        });
        containerLogs.value = (result as string).split("\n").filter((line) => line.trim() !== "");
    } catch (error) {
        ElMessage.error(`获取容器日志失败: ${error}`);
    } finally {
        isLoadingLogs.value = false;
    }
}

// 过滤日志的计算属性
const filteredLogs = computed(() => {
    if (!logSearchQuery.value) {
        return containerLogs.value;
    }
    return containerLogs.value.filter((log) =>
        log.toLowerCase().includes(logSearchQuery.value.toLowerCase()),
    );
});

// 容器终端相关方法
async function executeTerminalCommand() {
    if (!selectedContainer.value || !terminalCommand.value) return;

    try {
        const result = await invoke("exec_container_command", {
            containerId: selectedContainer.value.id,
            command: terminalCommand.value,
            shell: true,
        });
        terminalOutput.value.push(`$ ${terminalCommand.value}`);
        terminalOutput.value.push(
            ...(result as string).split("\n").filter((line) => line.trim() !== ""),
        );
        // 清空命令输入
        terminalCommand.value = "";
    } catch (error) {
        ElMessage.error(`执行命令失败: ${error}`);
    }
}

// 镜像管理相关方法
async function listImages() {
    try {
        const result = await invoke("list_images");
        images.value = result as any[];
    } catch (error) {
        ElMessage.error(`获取镜像列表失败: ${error}`);
    }
}

async function pullImage() {
    if (!newImageName.value) {
        ElMessage.warning("请输入镜像名称");
        return;
    }

    isPullingImage.value = true;

    try {
        await invoke("pull_image", { imageName: newImageName.value });
        ElMessage.success("镜像拉取成功！");
        await listImages();

        // 重置表单
        newImageName.value = "";
    } catch (error) {
        ElMessage.error(`拉取镜像失败: ${error}`);
    } finally {
        isPullingImage.value = false;
    }
}

async function deleteImage(imageId: string) {
    isManagingImage.value = true;
    try {
        await invoke("delete_image", { imageId });
        ElMessage.success("镜像删除成功！");
        await listImages();

        // 如果删除的是当前查看的镜像，返回镜像列表
        if (selectedImage.value && selectedImage.value.id === imageId) {
            switchTab("images");
        }
    } catch (error) {
        ElMessage.error(`删除镜像失败: ${error}`);
    } finally {
        isManagingImage.value = false;
    }
}

function showImageDetails(image: any) {
    selectedImage.value = image;
    currentTab.value = "image-details";
}

// 网络管理相关方法
async function listNetworks() {
    try {
        const result = await invoke("list_networks");
        // 转换网络信息结构以匹配前端期望的格式
        networks.value = (result as any[]).map((network) => ({
            id: network.name, // 使用名称作为 ID
            name: network.name,
            driver: network.network_type,
            scope: network.network_type === "bridge" ? "local" : network.network_type,
            enable_ipv6: false,
            internal: false,
            attachable: true,
            ingress: false,
            containers: {}, // 空容器映射
            options: {}, // 空选项映射
            labels: {}, // 空标签映射
        }));
    } catch (error) {
        ElMessage.error(`获取网络列表失败: ${error}`);
    }
}

async function createNetwork() {
    if (!newNetworkName.value) {
        ElMessage.warning("请输入网络名称");
        return;
    }

    isManagingNetwork.value = true;
    try {
        await invoke("create_network", {
            name: newNetworkName.value,
            driver: newNetworkDriver.value,
        });

        ElMessage.success("网络创建成功！");
        await listNetworks();

        // 重置表单
        newNetworkName.value = "";
        newNetworkDriver.value = "bridge";
    } catch (error) {
        ElMessage.error(`创建网络失败: ${error}`);
    } finally {
        isManagingNetwork.value = false;
    }
}

async function deleteNetwork(networkId: string) {
    isManagingNetwork.value = true;
    try {
        await invoke("delete_network", { networkId });
        ElMessage.success("网络删除成功！");
        await listNetworks();

        // 如果删除的是当前查看的网络，返回网络列表
        if (selectedNetwork.value && selectedNetwork.value.id === networkId) {
            switchTab("networks");
        }
    } catch (error) {
        ElMessage.error(`删除网络失败: ${error}`);
    } finally {
        isManagingNetwork.value = false;
    }
}

function showNetworkDetails(network: any) {
    selectedNetwork.value = network;
    currentTab.value = "network-details";
}

// 卷管理相关方法
async function listVolumes() {
    try {
        const result = await invoke("list_volumes");
        volumes.value = result as any[];
    } catch (error) {
        ElMessage.error(`获取卷列表失败: ${error}`);
    }
}

async function createVolume() {
    if (!newVolumeName.value) {
        ElMessage.warning("请输入卷名称");
        return;
    }

    isManagingVolume.value = true;
    try {
        await invoke("create_volume", {
            name: newVolumeName.value,
            driver: newVolumeDriver.value,
        });

        ElMessage.success("卷创建成功！");
        await listVolumes();

        // 重置表单
        newVolumeName.value = "";
        newVolumeDriver.value = "local";
    } catch (error) {
        ElMessage.error(`创建卷失败: ${error}`);
    } finally {
        isManagingVolume.value = false;
    }
}

async function deleteVolume(volumeId: string) {
    isManagingVolume.value = true;
    try {
        await invoke("delete_volume", { volumeId });
        ElMessage.success("卷删除成功！");
        await listVolumes();

        // 如果删除的是当前查看的卷，返回卷列表
        if (selectedVolume.value && selectedVolume.value.id === volumeId) {
            switchTab("volumes");
        }
    } catch (error) {
        ElMessage.error(`删除卷失败: ${error}`);
    } finally {
        isManagingVolume.value = false;
    }
}

function showVolumeDetails(volume: any) {
    selectedVolume.value = volume;
    currentTab.value = "volume-details";
}

// 系统状态相关方法
async function getSystemStatus() {
    try {
        const result = await invoke("get_system_status");
        systemStatus.value = result;
    } catch (error) {
        ElMessage.error(`获取系统状态失败: ${error}`);
    }
}

// Kubernetes 相关方法
async function listK8sPods() {
    try {
        // 模拟 Kubernetes Pods 列表
        k8sPods.value = [
            {
                metadata: {
                    name: "nginx-12345",
                    namespace: "default",
                },
                status: {
                    phase: "Running",
                },
            },
            {
                metadata: {
                    name: "redis-67890",
                    namespace: "default",
                },
                status: {
                    phase: "Running",
                },
            },
            {
                metadata: {
                    name: "mysql-54321",
                    namespace: "default",
                },
                status: {
                    phase: "Pending",
                },
            },
        ];
    } catch (error) {
        ElMessage.error(`获取 Kubernetes Pods 失败: ${error}`);
    }
}

async function listK8sServices() {
    try {
        // 模拟 Kubernetes 服务列表
        k8sServices.value = [
            {
                metadata: {
                    name: "nginx-service",
                },
                spec: {
                    type: "ClusterIP",
                    ports: [
                        {
                            port: 80,
                        },
                    ],
                },
            },
            {
                metadata: {
                    name: "redis-service",
                },
                spec: {
                    type: "ClusterIP",
                    ports: [
                        {
                            port: 6379,
                        },
                    ],
                },
            },
        ];
    } catch (error) {
        ElMessage.error(`获取 Kubernetes 服务失败: ${error}`);
    }
}

async function describeK8sPod(podName: string) {
    try {
        ElMessage.info(`查看 Pod ${podName} 详情`);
    } catch (error) {
        ElMessage.error(`查看 Pod 详情失败: ${error}`);
    }
}

async function describeK8sService(serviceName: string) {
    try {
        ElMessage.info(`查看服务 ${serviceName} 详情`);
    } catch (error) {
        ElMessage.error(`查看服务详情失败: ${error}`);
    }
}

// 工具函数
function formatSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function formatDate(timestamp: any): string {
    if (!timestamp) return "未知";
    const date = new Date(timestamp);
    return date.toLocaleString();
}

// 实时状态更新
function startStatusUpdates() {
    statusUpdateInterval = window.setInterval(async () => {
        await listContainers();
        await getSystemStatus();
    }, 5000); // 每5秒更新一次
}

function stopStatusUpdates() {
    if (statusUpdateInterval) {
        clearInterval(statusUpdateInterval);
        statusUpdateInterval = undefined;
    }
}

// 页面加载时自动初始化
onMounted(async () => {
    // 自动登录（模拟）
    isAuthenticated.value = true;

    // 尝试初始化 Docker（在后台执行）
    initDockerInBackground();
});

// 在后台初始化 Docker 服务
async function initDockerInBackground() {
    try {
        // 初始化 Docker 服务
        await invoke("init_docker");

        // 并行执行初始化操作，提高启动速度
        await Promise.all([
            listContainers(),
            listImages(),
            listNetworks(),
            listVolumes(),
            getSystemStatus(),
        ]);

        // 启动实时状态更新
        startStatusUpdates();
    } catch (error) {
        console.log("Docker 初始化失败，可能需要手动初始化:", error);
    }
}

// 页面卸载时清理
onUnmounted(() => {
    stopStatusUpdates();
});
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
</style>
