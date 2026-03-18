export interface Container {
    id: string;
    name: string;
    image: string;
    status: "Running" | "Exited" | "Paused" | string;
    created: string;
    ports?: string;
    cpuUsage?: string;
    memoryUsage?: string;
    networkUsage?: string;
    labels?: Record<string, string>;
    tags?: string[];
}

export interface Image {
    id: string;
    repoTags?: string[];
    size: string;
    created: string;
    architecture?: string;
    history?: {
        created?: string;
        createdBy?: string;
    }[];
    labels?: Record<string, string>;
    tags?: string[];
}

export interface Network {
    id: string;
    name: string;
    driver: string;
    scope: string;
    enable_ipv6: boolean;
    internal: boolean;
    attachable: boolean;
    ingress: boolean;
    containers?: {
        [key: string]: {
            name: string;
            mac_address: string;
            ipv4_address: string;
        };
    };
    labels?: Record<string, string>;
    tags?: string[];
}

export interface Volume {
    id: string;
    name: string;
    size: number;
    created_at: string;
    mount_point: string;
    driver: string;
    used_by: string[];
    labels?: Record<string, string>;
    tags?: string[];
}

export interface DockerSystemStatus {
    status: string;
    version: string;
    apiVersion: string;
    osType: string;
    architecture: string;
    kernelVersion: string;
    totalMemory: number;
    usedMemory: number;
    totalCPU: number;
    usedCPU: number;
}

export enum EndpointType {
    Local = "Local",
    Remote = "Remote",
    Cloud = "Cloud",
}

export enum EndpointStatus {
    Connected = "Connected",
    Connecting = "Connecting",
    Failed = "Failed",
    Disconnected = "Disconnected",
}

export interface EndpointConfig {
    id: string;
    name: string;
    endpoint_type: EndpointType;
    url: string;
    use_tls: boolean;
    tls_cert_path?: string;
    tls_key_path?: string;
    tls_ca_path?: string;
    auth_token?: string;
    labels: Record<string, string>;
}

export interface EndpointInfo {
    config: EndpointConfig;
    status: EndpointStatus;
    created_at: string;
    last_connected_at?: string;
    connection_info?: DockerSystemStatus;
}
