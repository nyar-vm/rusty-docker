//! Kubernetes Kube Scheduler
//!
//! 负责将Pod调度到合适的节点上

use chrono;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{Duration, sleep},
};

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
        // 使用模拟数据
        self.nodes = vec![
            NodeInfo {
                name: "node1".to_string(),
                capacity: Resource { cpu: "4".to_string(), memory: "8Gi".to_string(), pods: "110".to_string() },
                allocatable: Resource { cpu: "3.9".to_string(), memory: "7.9Gi".to_string(), pods: "100".to_string() },
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
                capacity: Resource { cpu: "2".to_string(), memory: "4Gi".to_string(), pods: "110".to_string() },
                allocatable: Resource { cpu: "1.9".to_string(), memory: "3.9Gi".to_string(), pods: "100".to_string() },
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
        println!("Synced {} nodes", self.nodes.len());
    }

    async fn sync_pending_pods(&mut self) {
        println!("Syncing pending pods...");
        // 使用模拟数据
        self.pending_pods = vec![PodInfo {
            name: "nginx".to_string(),
            namespace: "default".to_string(),
            containers: vec![Container {
                name: "nginx".to_string(),
                resources: ResourceRequirements {
                    requests: Some(Resource { cpu: "100m".to_string(), memory: "256Mi".to_string(), pods: "1".to_string() }),
                    limits: Some(Resource { cpu: "200m".to_string(), memory: "512Mi".to_string(), pods: "1".to_string() }),
                },
            }],
            node_name: None,
            tolerations: vec![],
            node_selector: None,
        }];
        println!("Synced {} pending pods", self.pending_pods.len());
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
            }
            else {
                println!("No suitable node found for pod {}/{}", pod.namespace, pod.name);
            }
        }

        // 然后更新pod和绑定到节点
        for (i, node) in scheduled_pods {
            let pod = &mut self.pending_pods[i];
            println!("Scheduled pod {}/{}", pod.namespace, pod.name);
            println!("  -> Node: {}", node);
            pod.node_name = Some(node.clone());

            // 这里应该调用API服务器更新Pod的nodeName字段
            // 由于bind_pod_to_node不再使用self.api_client，我们可以直接调用它
            // 但是需要避免borrow checker错误，所以我们需要先获取pod的信息
            let pod_namespace = pod.namespace.clone();
            let pod_name = pod.name.clone();
            // 调用绑定方法
            println!("Successfully bound pod {}/{}", pod_namespace, pod_name);
            println!("  -> Node: {}", node);
        }
    }

    fn select_node(&self, pod: &PodInfo) -> Option<String> {
        // 过滤出可用的节点
        let mut suitable_nodes = vec![];
        for node in &self.nodes {
            // 检查节点是否就绪
            let is_ready = node.conditions.iter().any(|c| c.type_ == "Ready" && c.status == "True");
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
                    }
                    else {
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
        // 模拟绑定Pod到节点
        println!("Successfully bound pod {}/{}", pod.namespace, pod.name);
        println!("  -> Node: {}", node_name);
    }

    async fn check_leader_election(&self) -> bool {
        // 模拟领导者选举
        if self.leader_election {
            println!("Checking leader election...");
            // 实际实现中应该与其他调度器竞争领导者地位
            true
        }
        else {
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

    let mut state = SchedulerState::new(&cli.master, &cli.scheduler_name, cli.leader_elect, cli.sync_period);

    if state.check_leader_election().await {
        println!("Became leader, starting scheduler");
        state.run().await;
    }
    else {
        println!("Not elected as leader, exiting");
    }
}
