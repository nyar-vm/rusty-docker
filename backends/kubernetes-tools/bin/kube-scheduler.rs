//! Kubernetes Kube Scheduler
//!
//! 负责将Pod调度到合适的节点上

use clap::Parser;
use docker_tools::create_base_command;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use wae_request::HttpClient;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// API server URL
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    master: String,

    /// Leader election enabled
    #[arg(long, default_value = "true")]
    leader_elect: bool,

    /// Scheduler name
    #[arg(long, default_value = "default-scheduler")]
    scheduler_name: String,

    /// Sync period
    #[arg(long, default_value = "30")]
    sync_period: u64,
}

/// 节点信息
struct NodeInfo {
    name: String,
    capacity: Resource,
    allocatable: Resource,
    labels: std::collections::HashMap<String, String>,
    taints: Vec<Taint>,
    conditions: Vec<NodeCondition>,
}

/// 资源信息
struct Resource {
    cpu: String,
    memory: String,
    pods: String,
}

/// 污点信息
struct Taint {
    key: String,
    value: String,
    effect: String,
}

/// 节点条件
struct NodeCondition {
    type_: String,
    status: String,
    last_heartbeat_time: chrono::DateTime<chrono::Utc>,
    last_transition_time: chrono::DateTime<chrono::Utc>,
    reason: String,
    message: String,
}

/// Pod信息
struct PodInfo {
    name: String,
    namespace: String,
    containers: Vec<Container>,
    node_name: Option<String>,
    tolerations: Vec<Toleration>,
    node_selector: Option<std::collections::HashMap<String, String>>,
}

/// 容器信息
struct Container {
    name: String,
    resources: ResourceRequirements,
}

/// 资源需求
struct ResourceRequirements {
    requests: Option<Resource>,
    limits: Option<Resource>,
}

/// 容忍度
struct Toleration {
    key: String,
    operator: String,
    value: String,
    effect: String,
}

/// 调度器状态
struct SchedulerState {
    api_client: HttpClient,
    master_url: String,
    scheduler_name: String,
    leader_election: bool,
    nodes: Vec<NodeInfo>,
    pending_pods: Vec<PodInfo>,
    sync_period: Duration,
}

impl SchedulerState {
    fn new(master: &str, scheduler_name: &str, leader_elect: bool, sync_period: u64) -> Self {
        Self {
            api_client: HttpClient::default(),
            master_url: master.to_string(),
            scheduler_name: scheduler_name.to_string(),
            leader_election: leader_elect,
            nodes: vec![],
            pending_pods: vec![],
            sync_period: Duration::from_secs(sync_period),
        }
    }

    async fn run(&mut self) {
        println!("Starting kube-scheduler");
        println!("Scheduler name: {}", self.scheduler_name);

        loop {
            self.sync_nodes().await;
            self.sync_pending_pods().await;
            self.schedule_pods().await;
            sleep(self.sync_period).await;
        }
    }

    async fn sync_nodes(&mut self) {
        println!("Syncing nodes...");
        // 从API服务器获取节点信息
        let url = format!("{}/api/v1/nodes", self.master_url);
        match self.api_client.get(&url).await {
            Ok(response) => {
                if response.is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(data) => {
                            self.nodes = vec![];
                            if let Some(items) = data.get("items") {
                                for node in items.as_array().unwrap() {
                                    let name =
                                        node["metadata"]["name"].as_str().unwrap().to_string();

                                    // 解析资源信息
                                    let capacity = Resource {
                                        cpu: node["status"]["capacity"]["cpu"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        memory: node["status"]["capacity"]["memory"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        pods: node["status"]["capacity"]["pods"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                    };

                                    let allocatable = Resource {
                                        cpu: node["status"]["allocatable"]["cpu"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        memory: node["status"]["allocatable"]["memory"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        pods: node["status"]["allocatable"]["pods"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                    };

                                    // 解析标签
                                    let mut labels = std::collections::HashMap::new();
                                    if let Some(node_labels) = node["metadata"].get("labels") {
                                        for (key, value) in node_labels.as_object().unwrap() {
                                            labels.insert(
                                                key.clone(),
                                                value.as_str().unwrap().to_string(),
                                            );
                                        }
                                    }

                                    // 解析污点
                                    let mut taints = vec![];
                                    if let Some(node_taints) = node["spec"].get("taints") {
                                        for taint in node_taints.as_array().unwrap() {
                                            taints.push(Taint {
                                                key: taint["key"].as_str().unwrap().to_string(),
                                                value: taint["value"].as_str().unwrap().to_string(),
                                                effect: taint["effect"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                            });
                                        }
                                    }

                                    // 解析节点条件
                                    let mut conditions = vec![];
                                    if let Some(node_conditions) = node["status"].get("conditions")
                                    {
                                        for condition in node_conditions.as_array().unwrap() {
                                            conditions.push(NodeCondition {
                                                type_: condition["type"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                status: condition["status"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                last_heartbeat_time: chrono::Utc::now(),
                                                last_transition_time: chrono::Utc::now(),
                                                reason: condition["reason"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                message: condition["message"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                            });
                                        }
                                    }

                                    self.nodes.push(NodeInfo {
                                        name,
                                        capacity,
                                        allocatable,
                                        labels,
                                        taints,
                                        conditions,
                                    });
                                }
                            }
                            println!("Synced {} nodes", self.nodes.len());
                        }
                        Err(e) => {
                            eprintln!("Error parsing nodes: {:?}", e);
                        }
                    }
                } else {
                    eprintln!("Error getting nodes: {:?}", response.status);
                }
            }
            Err(e) => {
                eprintln!("Error requesting nodes: {:?}", e);
                // 使用模拟数据
                self.nodes = vec![
                    NodeInfo {
                        name: "node1".to_string(),
                        capacity: Resource {
                            cpu: "4".to_string(),
                            memory: "8Gi".to_string(),
                            pods: "110".to_string(),
                        },
                        allocatable: Resource {
                            cpu: "3.9".to_string(),
                            memory: "7.9Gi".to_string(),
                            pods: "100".to_string(),
                        },
                        labels: std::collections::HashMap::from([
                            ("kubernetes.io/hostname".to_string(), "node1".to_string()),
                            ("kubernetes.io/os".to_string(), "linux".to_string()),
                        ]),
                        taints: vec![],
                        conditions: vec![NodeCondition {
                            type_: "Ready".to_string(),
                            status: "True".to_string(),
                            last_heartbeat_time: chrono::Utc::now(),
                            last_transition_time: chrono::Utc::now(),
                            reason: "KubeletReady".to_string(),
                            message: "kubelet is ready".to_string(),
                        }],
                    },
                    NodeInfo {
                        name: "node2".to_string(),
                        capacity: Resource {
                            cpu: "2".to_string(),
                            memory: "4Gi".to_string(),
                            pods: "110".to_string(),
                        },
                        allocatable: Resource {
                            cpu: "1.9".to_string(),
                            memory: "3.9Gi".to_string(),
                            pods: "100".to_string(),
                        },
                        labels: std::collections::HashMap::from([
                            ("kubernetes.io/hostname".to_string(), "node2".to_string()),
                            ("kubernetes.io/os".to_string(), "linux".to_string()),
                        ]),
                        taints: vec![],
                        conditions: vec![NodeCondition {
                            type_: "Ready".to_string(),
                            status: "True".to_string(),
                            last_heartbeat_time: chrono::Utc::now(),
                            last_transition_time: chrono::Utc::now(),
                            reason: "KubeletReady".to_string(),
                            message: "kubelet is ready".to_string(),
                        }],
                    },
                ];
            }
        }
    }

    async fn sync_pending_pods(&mut self) {
        println!("Syncing pending pods...");
        // 从API服务器获取待调度的Pod
        let url = format!("{}/api/v1/pods", self.master_url);
        match self.api_client.get(&url).await {
            Ok(response) => {
                if response.is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(data) => {
                            self.pending_pods = vec![];
                            if let Some(items) = data.get("items") {
                                for pod in items.as_array().unwrap() {
                                    // 只处理未调度的Pod
                                    if pod["spec"].get("nodeName").is_none() {
                                        let name =
                                            pod["metadata"]["name"].as_str().unwrap().to_string();
                                        let namespace = pod["metadata"]["namespace"]
                                            .as_str()
                                            .unwrap()
                                            .to_string();

                                        // 解析容器信息
                                        let mut containers = vec![];
                                        if let Some(pod_containers) = pod["spec"].get("containers")
                                        {
                                            for container in pod_containers.as_array().unwrap() {
                                                let container_name =
                                                    container["name"].as_str().unwrap().to_string();

                                                // 解析资源需求
                                                let mut requests = None;
                                                let mut limits = None;
                                                if let Some(resources) = container.get("resources")
                                                {
                                                    if let Some(requests_obj) =
                                                        resources.get("requests")
                                                    {
                                                        requests = Some(Resource {
                                                            cpu: requests_obj["cpu"]
                                                                .as_str()
                                                                .unwrap_or("0")
                                                                .to_string(),
                                                            memory: requests_obj["memory"]
                                                                .as_str()
                                                                .unwrap_or("0")
                                                                .to_string(),
                                                            pods: "1".to_string(),
                                                        });
                                                    }
                                                    if let Some(limits_obj) =
                                                        resources.get("limits")
                                                    {
                                                        limits = Some(Resource {
                                                            cpu: limits_obj["cpu"]
                                                                .as_str()
                                                                .unwrap_or("0")
                                                                .to_string(),
                                                            memory: limits_obj["memory"]
                                                                .as_str()
                                                                .unwrap_or("0")
                                                                .to_string(),
                                                            pods: "1".to_string(),
                                                        });
                                                    }
                                                }

                                                containers.push(Container {
                                                    name: container_name,
                                                    resources: ResourceRequirements {
                                                        requests,
                                                        limits,
                                                    },
                                                });
                                            }
                                        }

                                        // 解析容忍度
                                        let mut tolerations = vec![];
                                        if let Some(pod_tolerations) =
                                            pod["spec"].get("tolerations")
                                        {
                                            for toleration in pod_tolerations.as_array().unwrap() {
                                                tolerations.push(Toleration {
                                                    key: toleration["key"]
                                                        .as_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    operator: toleration["operator"]
                                                        .as_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    value: toleration["value"]
                                                        .as_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    effect: toleration["effect"]
                                                        .as_str()
                                                        .unwrap()
                                                        .to_string(),
                                                });
                                            }
                                        }

                                        // 解析节点选择器
                                        let mut node_selector = None;
                                        if let Some(selector) = pod["spec"].get("nodeSelector") {
                                            let mut selector_map = std::collections::HashMap::new();
                                            for (key, value) in selector.as_object().unwrap() {
                                                selector_map.insert(
                                                    key.clone(),
                                                    value.as_str().unwrap().to_string(),
                                                );
                                            }
                                            node_selector = Some(selector_map);
                                        }

                                        self.pending_pods.push(PodInfo {
                                            name,
                                            namespace,
                                            containers,
                                            node_name: None,
                                            tolerations,
                                            node_selector,
                                        });
                                    }
                                }
                            }
                            println!("Synced {} pending pods", self.pending_pods.len());
                        }
                        Err(e) => {
                            eprintln!("Error parsing pods: {:?}", e);
                        }
                    }
                } else {
                    eprintln!("Error getting pods: {:?}", response.status);
                }
            }
            Err(e) => {
                eprintln!("Error requesting pods: {:?}", e);
                // 使用模拟数据
                self.pending_pods = vec![PodInfo {
                    name: "nginx".to_string(),
                    namespace: "default".to_string(),
                    containers: vec![Container {
                        name: "nginx".to_string(),
                        resources: ResourceRequirements {
                            requests: Some(Resource {
                                cpu: "100m".to_string(),
                                memory: "256Mi".to_string(),
                                pods: "1".to_string(),
                            }),
                            limits: Some(Resource {
                                cpu: "200m".to_string(),
                                memory: "512Mi".to_string(),
                                pods: "1".to_string(),
                            }),
                        },
                    }],
                    node_name: None,
                    tolerations: vec![],
                    node_selector: None,
                }];
            }
        }
    }

    async fn schedule_pods(&mut self) {
        println!("Scheduling pods...");
        // 调度过程
        let pod_count = self.pending_pods.len();
        let mut scheduled_pods = vec![];
        
        // 先收集所有需要调度的pod和对应的节点
        for i in 0..pod_count {
            let pod = &self.pending_pods[i];
            if let Some(node) = self.select_node(pod) {
                scheduled_pods.push((i, node));
            } else {
                println!(
                    "No suitable node found for pod {}/{}",
                    pod.namespace, pod.name
                );
            }
        }
        
        // 然后更新pod和绑定到节点
        for (i, node) in scheduled_pods {
            let pod = &mut self.pending_pods[i];
            println!("Scheduled pod {}/{}", pod.namespace, pod.name, node);
            pod.node_name = Some(node.clone());
            
            // 这里应该调用API服务器更新Pod的nodeName字段
            self.bind_pod_to_node(pod, &node).await;
        }
    }

    fn select_node(&self, pod: &PodInfo) -> Option<String> {
        // 过滤出可用的节点
        let mut suitable_nodes = vec![];
        for node in &self.nodes {
            // 检查节点是否就绪
            let is_ready = node
                .conditions
                .iter()
                .any(|c| c.type_ == "Ready" && c.status == "True");
            if !is_ready {
                continue;
            }

            // 检查节点选择器
            if let Some(selector) = &pod.node_selector {
                let mut match_selector = true;
                for (key, value) in selector {
                    if let Some(node_value) = node.labels.get(key) {
                        if node_value != value {
                            match_selector = false;
                            break;
                        }
                    } else {
                        match_selector = false;
                        break;
                    }
                }
                if !match_selector {
                    continue;
                }
            }

            // 检查污点容忍度
            let mut tolerate_taints = true;
            for taint in &node.taints {
                let mut tolerated = false;
                for toleration in &pod.tolerations {
                    if toleration.key == taint.key && toleration.effect == taint.effect {
                        if toleration.operator == "Exists" || toleration.value == taint.value {
                            tolerated = true;
                            break;
                        }
                    }
                }
                if !tolerated {
                    tolerate_taints = false;
                    break;
                }
            }
            if !tolerate_taints {
                continue;
            }

            // 这里可以添加更多的调度策略，如资源需求检查等
            suitable_nodes.push(node.name.clone());
        }

        // 选择第一个合适的节点
        suitable_nodes.first().cloned()
    }

    async fn bind_pod_to_node(&self, pod: &PodInfo, node_name: &str) {
        // 调用API服务器绑定Pod到节点
        let url = format!(
            "{}/api/v1/namespaces/{}/pods/{}/binding",
            self.master_url, pod.namespace, pod.name
        );

        let binding = serde_json::json!({
            "apiVersion": "v1",
            "kind": "Binding",
            "metadata": {
                "name": pod.name,
                "namespace": pod.namespace
            },
            "target": {
                "apiVersion": "v1",
                "kind": "Node",
                "name": node_name
            }
        });

        match self.api_client.post_json(&url, &binding).await {
            Ok(response) => {
                if response.is_success() {
                    println!("Successfully bound pod {}/{}", pod.namespace, pod.name);
                } else {
                    eprintln!(
                        "Error binding pod {}/{}: {:?}",
                        pod.namespace, pod.name, response.status
                    );
                }
            }
            Err(e) => {
                eprintln!("Error binding pod {}/{}: {:?}", pod.namespace, pod.name, e);
            }
        }
    }

    async fn check_leader_election(&self) -> bool {
        // 模拟领导者选举
        if self.leader_election {
            println!("Checking leader election...");
            // 实际实现中应该与其他调度器竞争领导者地位
            true
        } else {
            true
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("Starting Kubernetes Kube Scheduler");
    println!("Master: {}", cli.master);
    println!("Scheduler name: {}", cli.scheduler_name);
    println!("Leader election: {}", cli.leader_elect);
    println!("Sync period: {}s", cli.sync_period);

    let mut state = SchedulerState::new(
        &cli.master,
        &cli.scheduler_name,
        cli.leader_elect,
        cli.sync_period,
    );

    if state.check_leader_election().await {
        println!("Became leader, starting scheduler");
        state.run().await;
    } else {
        println!("Not elected as leader, exiting");
    }
}
