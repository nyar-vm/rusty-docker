<template>
  <div class="monitor-container">
    <h1 class="text-2xl font-bold mb-6">系统监控</h1>
    
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- CPU 监控 -->
      <div class="bg-white rounded-lg shadow p-4">
        <h2 class="text-lg font-semibold mb-4">CPU 使用率</h2>
        <div class="h-64">
          <canvas ref="cpuChart"></canvas>
        </div>
        <div class="mt-4 flex justify-between">
          <div>
            <span class="text-sm text-gray-500">当前使用率:</span>
            <span class="ml-2 font-medium">{{ cpuUsage }}%</span>
          </div>
          <div>
            <span class="text-sm text-gray-500">阈值:</span>
            <span class="ml-2 font-medium">{{ config.cpu_threshold }}%</span>
          </div>
        </div>
      </div>

      <!-- 内存监控 -->
      <div class="bg-white rounded-lg shadow p-4">
        <h2 class="text-lg font-semibold mb-4">内存使用率</h2>
        <div class="h-64">
          <canvas ref="memoryChart"></canvas>
        </div>
        <div class="mt-4 flex justify-between">
          <div>
            <span class="text-sm text-gray-500">当前使用率:</span>
            <span class="ml-2 font-medium">{{ memoryUsage }}%</span>
          </div>
          <div>
            <span class="text-sm text-gray-500">阈值:</span>
            <span class="ml-2 font-medium">{{ config.memory_threshold }}%</span>
          </div>
        </div>
      </div>

      <!-- 网络监控 -->
      <div class="bg-white rounded-lg shadow p-4">
        <h2 class="text-lg font-semibold mb-4">网络流量</h2>
        <div class="h-64">
          <canvas ref="networkChart"></canvas>
        </div>
        <div class="mt-4 flex justify-between">
          <div>
            <span class="text-sm text-gray-500">发送:</span>
            <span class="ml-2 font-medium">{{ formatBytes(networkSent) }}</span>
          </div>
          <div>
            <span class="text-sm text-gray-500">接收:</span>
            <span class="ml-2 font-medium">{{ formatBytes(networkRecv) }}</span>
          </div>
        </div>
      </div>

      <!-- 磁盘监控 -->
      <div class="bg-white rounded-lg shadow p-4">
        <h2 class="text-lg font-semibold mb-4">磁盘使用率</h2>
        <div class="h-64">
          <canvas ref="diskChart"></canvas>
        </div>
        <div class="mt-4 flex justify-between">
          <div>
            <span class="text-sm text-gray-500">当前使用率:</span>
            <span class="ml-2 font-medium">{{ diskUsage }}%</span>
          </div>
          <div>
            <span class="text-sm text-gray-500">阈值:</span>
            <span class="ml-2 font-medium">{{ config.disk_threshold }}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 告警信息 -->
    <div class="mt-8 bg-white rounded-lg shadow p-4">
      <h2 class="text-lg font-semibold mb-4">告警信息</h2>
      <div v-if="alerts.length === 0" class="text-center text-gray-500 py-4">
        暂无告警信息
      </div>
      <div v-else class="space-y-2">
        <div 
          v-for="alert in alerts" 
          :key="alert.id"
          class="p-3 border-l-4 border-yellow-500 bg-yellow-50"
        >
          <div class="flex justify-between">
            <div class="font-medium">{{ alert.message }}</div>
            <div class="text-sm text-gray-500">{{ formatTime(alert.timestamp) }}</div>
          </div>
          <div class="text-sm text-gray-600 mt-1">
            资源: {{ alert.resource }} | 当前值: {{ alert.current_value.toFixed(2) }} | 阈值: {{ alert.threshold.toFixed(2) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Chart from "chart.js/auto";

// 状态
const cpuChart = ref<HTMLCanvasElement | null>(null);
const memoryChart = ref<HTMLCanvasElement | null>(null);
const networkChart = ref<HTMLCanvasElement | null>(null);
const diskChart = ref<HTMLCanvasElement | null>(null);

const cpuUsage = ref(0);
const memoryUsage = ref(0);
const networkSent = ref(0);
const networkRecv = ref(0);
const diskUsage = ref(0);

const resources = ref<any[]>([]);
const alerts = ref<any[]>([]);
const config = ref({
    cpu_threshold: 80,
    memory_threshold: 80,
    disk_threshold: 80,
    network_threshold: 10 * 1024 * 1024,
});

// Chart 实例
let cpuChartInstance: Chart | null = null;
let memoryChartInstance: Chart | null = null;
let networkChartInstance: Chart | null = null;
let diskChartInstance: Chart | null = null;

// 格式化函数
const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

const formatTime = (timestamp: string): string => {
    return new Date(timestamp).toLocaleString();
};

// 初始化图表
const initCharts = () => {
    // CPU 图表
    if (cpuChart.value) {
        cpuChartInstance = new Chart(cpuChart.value, {
            type: "line",
            data: {
                labels: [],
                datasets: [
                    {
                        label: "CPU 使用率 (%)",
                        data: [],
                        borderColor: "rgb(75, 192, 192)",
                        tension: 0.1,
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100,
                    },
                },
            },
        });
    }

    // 内存图表
    if (memoryChart.value) {
        memoryChartInstance = new Chart(memoryChart.value, {
            type: "line",
            data: {
                labels: [],
                datasets: [
                    {
                        label: "内存使用率 (%)",
                        data: [],
                        borderColor: "rgb(255, 99, 132)",
                        tension: 0.1,
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100,
                    },
                },
            },
        });
    }

    // 网络图表
    if (networkChart.value) {
        networkChartInstance = new Chart(networkChart.value, {
            type: "line",
            data: {
                labels: [],
                datasets: [
                    {
                        label: "发送 (MB)",
                        data: [],
                        borderColor: "rgb(54, 162, 235)",
                        tension: 0.1,
                    },
                    {
                        label: "接收 (MB)",
                        data: [],
                        borderColor: "rgb(255, 206, 86)",
                        tension: 0.1,
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
            },
        });
    }

    // 磁盘图表
    if (diskChart.value) {
        diskChartInstance = new Chart(diskChart.value, {
            type: "line",
            data: {
                labels: [],
                datasets: [
                    {
                        label: "磁盘使用率 (%)",
                        data: [],
                        borderColor: "rgb(153, 102, 255)",
                        tension: 0.1,
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100,
                    },
                },
            },
        });
    }
};

// 更新图表数据
const updateCharts = () => {
    if (resources.value.length === 0) return;

    const labels = resources.value.map((r) => new Date(r.timestamp).toLocaleTimeString());
    const cpuData = resources.value.map((r) => r.cpu.total);
    const memoryData = resources.value.map((r) => r.memory.percent);
    const networkSentData = resources.value.map((r) => r.network.bytes_sent / (1024 * 1024));
    const networkRecvData = resources.value.map((r) => r.network.bytes_recv / (1024 * 1024));
    const diskData = resources.value.map((r) => r.disk.percent);

    // 更新 CPU 图表
    if (cpuChartInstance) {
        cpuChartInstance.data.labels = labels;
        cpuChartInstance.data.datasets[0].data = cpuData;
        cpuChartInstance.update();
    }

    // 更新内存图表
    if (memoryChartInstance) {
        memoryChartInstance.data.labels = labels;
        memoryChartInstance.data.datasets[0].data = memoryData;
        memoryChartInstance.update();
    }

    // 更新网络图表
    if (networkChartInstance) {
        networkChartInstance.data.labels = labels;
        networkChartInstance.data.datasets[0].data = networkSentData;
        networkChartInstance.data.datasets[1].data = networkRecvData;
        networkChartInstance.update();
    }

    // 更新磁盘图表
    if (diskChartInstance) {
        diskChartInstance.data.labels = labels;
        diskChartInstance.data.datasets[0].data = diskData;
        diskChartInstance.update();
    }
};

// 获取监控数据
const fetchData = async () => {
    try {
        // 使用模拟数据替代 docker-monitor API
        const mockData = {
            resources: [
                {
                    timestamp: new Date().toISOString(),
                    cpu: { user: 30.0, system: 10.0, idle: 60.0, total: 40.0 },
                    memory: { total: 8 * 1024 * 1024 * 1024, used: 4 * 1024 * 1024 * 1024, free: 4 * 1024 * 1024 * 1024, percent: 50.0 },
                    network: { bytes_sent: 1024 * 1024, bytes_recv: 2048 * 1024, packets_sent: 1000, packets_recv: 2000, errors_sent: 0, errors_recv: 0 },
                    disk: { total: 100 * 1024 * 1024 * 1024, used: 50 * 1024 * 1024 * 1024, free: 50 * 1024 * 1024 * 1024, percent: 50.0 }
                }
            ],
            alerts: [],
            config: {
                cpu_threshold: 80.0,
                memory_threshold: 80.0,
                disk_threshold: 80.0,
                network_threshold: 10 * 1024 * 1024
            }
        };

        resources.value = mockData.resources;
        alerts.value = mockData.alerts;
        config.value = mockData.config;

        // 更新当前值
        if (mockData.resources.length > 0) {
            const last = mockData.resources[0];
            cpuUsage.value = last.cpu.total;
            memoryUsage.value = last.memory.percent;
            networkSent.value = last.network.bytes_sent;
            networkRecv.value = last.network.bytes_recv;
            diskUsage.value = last.disk.percent;
        }

        updateCharts();
    } catch (error) {
        console.error("获取监控数据失败:", error);
    }
};

// 定时获取数据
let interval: number | undefined;

onMounted(() => {
    initCharts();
    fetchData();
    interval = window.setInterval(fetchData, 5000);
});

onUnmounted(() => {
    if (interval) {
        clearInterval(interval);
    }
    // 销毁图表实例
    cpuChartInstance?.destroy();
    memoryChartInstance?.destroy();
    networkChartInstance?.destroy();
    diskChartInstance?.destroy();
});
</script>

<style scoped>
.monitor-container {
  padding: 20px;
}
</style>
