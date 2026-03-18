import { EndpointInfo, EndpointType } from "../../types/docker";

// 模拟端点数据
const mockEndpoints: EndpointInfo[] = [
    {
        config: {
            id: "1",
            name: "Local Docker",
            endpoint_type: EndpointType.Local,
            url: "unix:///var/run/docker.sock",
            use_tls: false,
            labels: {},
        },
        status: "Connected" as any,
        created_at: new Date().toISOString(),
        last_connected_at: new Date().toISOString(),
        connection_info: {
            status: "running",
            version: "20.10.21",
            apiVersion: "1.41",
            osType: "linux",
            architecture: "x86_64",
            kernelVersion: "5.15.0-56-generic",
            totalMemory: 16384,
            usedMemory: 8192,
            totalCPU: 8,
            usedCPU: 40,
        },
    },
    {
        config: {
            id: "2",
            name: "Remote Docker",
            endpoint_type: EndpointType.Remote,
            url: "tcp://192.168.1.100:2375",
            use_tls: false,
            labels: {},
        },
        status: "Disconnected" as any,
        created_at: new Date().toISOString(),
        last_connected_at: undefined,
        connection_info: undefined,
    },
];

// 获取所有端点
export const getEndpoints = async (): Promise<EndpointInfo[]> => {
    // 模拟 API 调用
    return new Promise((resolve) => {
        setTimeout(() => {
            resolve(mockEndpoints);
        }, 500);
    });
};

// 获取单个端点
export const getEndpoint = async (id: string): Promise<EndpointInfo> => {
    // 模拟 API 调用
    return new Promise((resolve, reject) => {
        setTimeout(() => {
            const endpoint = mockEndpoints.find((e) => e.config.id === id);
            if (endpoint) {
                resolve(endpoint);
            } else {
                reject(new Error("Endpoint not found"));
            }
        }, 300);
    });
};

// 创建端点
export const createEndpoint = async (
    endpoint: Omit<EndpointInfo["config"], "id">,
): Promise<EndpointInfo> => {
    // 模拟 API 调用
    return new Promise((resolve) => {
        setTimeout(() => {
            const newEndpoint: EndpointInfo = {
                config: {
                    ...endpoint,
                    id: Date.now().toString(),
                },
                status: "Disconnected" as any,
                created_at: new Date().toISOString(),
                last_connected_at: undefined,
                connection_info: undefined,
            };
            mockEndpoints.push(newEndpoint);
            resolve(newEndpoint);
        }, 500);
    });
};

// 更新端点
export const updateEndpoint = async (
    id: string,
    endpoint: Partial<EndpointInfo["config"]>,
): Promise<EndpointInfo> => {
    // 模拟 API 调用
    return new Promise((resolve, reject) => {
        setTimeout(() => {
            const index = mockEndpoints.findIndex((e) => e.config.id === id);
            if (index !== -1) {
                mockEndpoints[index] = {
                    ...mockEndpoints[index],
                    config: {
                        ...mockEndpoints[index].config,
                        ...endpoint,
                    },
                };
                resolve(mockEndpoints[index]);
            } else {
                reject(new Error("Endpoint not found"));
            }
        }, 500);
    });
};

// 删除端点
export const deleteEndpoint = async (id: string): Promise<void> => {
    // 模拟 API 调用
    return new Promise((resolve, reject) => {
        setTimeout(() => {
            const index = mockEndpoints.findIndex((e) => e.config.id === id);
            if (index !== -1) {
                mockEndpoints.splice(index, 1);
                resolve();
            } else {
                reject(new Error("Endpoint not found"));
            }
        }, 300);
    });
};

// 测试端点连接
export const testEndpointConnection = async (_id: string): Promise<string> => {
    // 模拟 API 调用
    return new Promise((resolve) => {
        setTimeout(() => {
            resolve("Connected");
        }, 1000);
    });
};
