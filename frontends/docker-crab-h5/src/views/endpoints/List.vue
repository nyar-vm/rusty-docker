<template>
  <div class="container mx-auto px-4 py-6">
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-gray-800">端点管理</h1>
      <button 
        @click="showAddModal = true"
        class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-lg transition-colors"
      >
        添加端点
      </button>
    </div>

    <!-- 端点列表 -->
    <div v-if="!endpointStore.loading" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div 
        v-for="endpoint in endpointStore.endpoints" 
        :key="endpoint.config.id"
        class="bg-white rounded-lg shadow-md p-4 border-l-4" 
        :class="{
          'border-green-500': endpoint.status === 'Connected',
          'border-yellow-500': endpoint.status === 'Connecting',
          'border-red-500': endpoint.status === 'Failed',
          'border-gray-500': endpoint.status === 'Disconnected'
        }"
      >
        <div class="flex justify-between items-start">
          <div>
            <h2 class="text-lg font-semibold text-gray-800">{{ endpoint.config.name }}</h2>
            <p class="text-sm text-gray-500 mb-2">{{ endpoint.config.url }}</p>
            <div class="flex items-center mb-2">
              <span 
                class="inline-block px-2 py-1 rounded-full text-xs font-medium"
                :class="{
                  'bg-green-100 text-green-800': endpoint.status === 'Connected',
                  'bg-yellow-100 text-yellow-800': endpoint.status === 'Connecting',
                  'bg-red-100 text-red-800': endpoint.status === 'Failed',
                  'bg-gray-100 text-gray-800': endpoint.status === 'Disconnected'
                }"
              >
                {{ endpoint.status }}
              </span>
              <span class="ml-2 text-xs text-gray-500">
                {{ endpoint.config.endpoint_type }}
              </span>
            </div>
            <div v-if="endpoint.last_connected_at" class="text-xs text-gray-500">
              最后连接: {{ formatDate(endpoint.last_connected_at) }}
            </div>
          </div>
          <div class="flex space-x-2">
            <button 
              @click="testConnection(endpoint.config.id)"
              class="p-2 rounded-md hover:bg-gray-100 text-gray-600"
              title="测试连接"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </button>
            <button 
              @click="editEndpoint(endpoint)"
              class="p-2 rounded-md hover:bg-gray-100 text-gray-600"
              title="编辑端点"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
              </svg>
            </button>
            <button 
              @click="confirmDeleteEndpoint(endpoint.config.id)"
              class="p-2 rounded-md hover:bg-gray-100 text-red-600"
              title="删除端点"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
        <div v-if="endpoint.connection_info" class="mt-4 pt-4 border-t border-gray-200">
          <h3 class="text-sm font-medium text-gray-700 mb-2">连接信息</h3>
          <div class="grid grid-cols-2 gap-2 text-xs text-gray-600">
            <div>版本: {{ endpoint.connection_info.version }}</div>
            <div>API版本: {{ endpoint.connection_info.apiVersion }}</div>
            <div>OS: {{ endpoint.connection_info.osType }}</div>
            <div>架构: {{ endpoint.connection_info.architecture }}</div>
            <div>内存: {{ (endpoint.connection_info.usedMemory / 1024).toFixed(1) }}GB / {{ (endpoint.connection_info.totalMemory / 1024).toFixed(1) }}GB</div>
            <div>CPU: {{ endpoint.connection_info.usedCPU }}%</div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="flex justify-center items-center h-32">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
    </div>

    <!-- 添加/编辑端点模态框 -->
    <div v-if="showAddModal || showEditModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl p-6 max-w-md w-full">
        <h2 class="text-xl font-bold text-gray-800 mb-4">
          {{ showEditModal ? '编辑端点' : '添加端点' }}
        </h2>
        <form @submit.prevent="handleSubmit">
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-1">名称</label>
            <input 
              v-model="form.name"
              type="text" 
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-1">类型</label>
            <select 
              v-model="form.endpoint_type"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            >
              <option value="Local">本地</option>
              <option value="Remote">远程</option>
              <option value="Cloud">云</option>
            </select>
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-1">URL</label>
            <input 
              v-model="form.url"
              type="text" 
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div class="mb-4">
            <label class="flex items-center">
              <input 
                v-model="form.use_tls"
                type="checkbox" 
                class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <span class="ml-2 text-sm text-gray-700">使用 TLS</span>
            </label>
          </div>
          <div v-if="form.use_tls" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">TLS 证书路径</label>
              <input 
                v-model="form.tls_cert_path"
                type="text" 
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">TLS 密钥路径</label>
              <input 
                v-model="form.tls_key_path"
                type="text" 
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">TLS CA 证书路径</label>
              <input 
                v-model="form.tls_ca_path"
                type="text" 
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-1">认证令牌</label>
            <input 
              v-model="form.auth_token"
              type="text" 
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div class="flex justify-end space-x-3 mt-6">
            <button 
              @click="closeModal"
              class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
            >
              取消
            </button>
            <button 
              type="submit"
              class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600"
            >
              {{ showEditModal ? '更新' : '添加' }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- 删除确认模态框 -->
    <div v-if="showDeleteModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl p-6 max-w-md w-full">
        <h2 class="text-xl font-bold text-gray-800 mb-4">确认删除</h2>
        <p class="text-gray-600 mb-6">确定要删除这个端点吗？此操作无法撤销。</p>
        <div class="flex justify-end space-x-3">
          <button 
            @click="showDeleteModal = false"
            class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
          >
            取消
          </button>
          <button 
            @click="deleteEndpoint"
            class="px-4 py-2 bg-red-500 text-white rounded-md hover:bg-red-600"
          >
            删除
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useEndpointStore } from "../../stores/modules/endpointStore";
import { EndpointType } from "../../types/docker";

const endpointStore = useEndpointStore();

// 模态框状态
const showAddModal = ref(false);
const showEditModal = ref(false);
const showDeleteModal = ref(false);

// 表单数据
const form = ref({
    name: "",
    endpoint_type: EndpointType.Local,
    url: "",
    use_tls: false,
    tls_cert_path: "",
    tls_key_path: "",
    tls_ca_path: "",
    auth_token: "",
    labels: {},
});

// 待删除的端点 ID
const deleteEndpointId = ref("");

// 待编辑的端点
const editingEndpoint = ref<any>(null);

// 格式化日期
const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString();
};

// 加载端点列表
onMounted(() => {
    endpointStore.fetchEndpoints();
});

// 打开添加模态框
// const openAddModal = () => {
//   form.value = {
//     name: '',
//     endpoint_type: EndpointType.Local,
//     url: '',
//     use_tls: false,
//     tls_cert_path: '',
//     tls_key_path: '',
//     tls_ca_path: '',
//     auth_token: '',
//     labels: {}
//   };
//   addModal.value.show();
// };

// 打开编辑模态框
const editEndpoint = (endpoint: any) => {
    editingEndpoint.value = endpoint;
    form.value = {
        name: endpoint.config.name,
        endpoint_type: endpoint.config.endpoint_type,
        url: endpoint.config.url,
        use_tls: endpoint.config.use_tls,
        tls_cert_path: endpoint.config.tls_cert_path || "",
        tls_key_path: endpoint.config.tls_key_path || "",
        tls_ca_path: endpoint.config.tls_ca_path || "",
        auth_token: endpoint.config.auth_token || "",
        labels: endpoint.config.labels || {},
    };
    showEditModal.value = true;
    showAddModal.value = false;
};

// 关闭模态框
const closeModal = () => {
    showAddModal.value = false;
    showEditModal.value = false;
    showDeleteModal.value = false;
};

// 处理表单提交
const handleSubmit = async () => {
    try {
        if (showEditModal.value && editingEndpoint.value) {
            await endpointStore.updateEndpoint(editingEndpoint.value.config.id, form.value);
        } else {
            await endpointStore.addEndpoint(form.value);
        }
        closeModal();
    } catch (error) {
        console.error("Error submitting form:", error);
    }
};

// 确认删除
const confirmDeleteEndpoint = (id: string) => {
    deleteEndpointId.value = id;
    showDeleteModal.value = true;
};

// 删除端点
const deleteEndpoint = async () => {
    try {
        await endpointStore.removeEndpoint(deleteEndpointId.value);
        showDeleteModal.value = false;
    } catch (error) {
        console.error("Error deleting endpoint:", error);
    }
};

// 测试连接
const testConnection = async (id: string) => {
    try {
        await endpointStore.testConnection(id);
    } catch (error) {
        console.error("Error testing connection:", error);
    }
};
</script>

<style scoped>
/* 自定义样式 */
</style>
