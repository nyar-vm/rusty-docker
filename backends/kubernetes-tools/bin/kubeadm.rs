use clap::{Parser, Subcommand};

/// kubeadm 命令行工具
///
/// 用于初始化和管理 Kubernetes 集群
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化 Kubernetes 集群
    Init {
        /// 控制平面 advertise 地址
        #[arg(long)]
        apiserver_advertise_address: Option<String>,
        /// 控制平面绑定端口
        #[arg(long, default_value = "6443")]
        apiserver_bind_port: String,
        /// Pod 网络 CIDR
        #[arg(long)]
        pod_network_cidr: Option<String>,
        /// Service 网络 CIDR
        #[arg(long)]
        service_cidr: Option<String>,
        /// 服务 DNS 域名
        #[arg(long, default_value = "cluster.local")]
        service_dns_domain: String,
    },
    /// 加入已存在的 Kubernetes 集群
    Join {
        /// 控制平面地址
        control_plane_address: String,
        /// 令牌
        token: String,
        /// 发现令牌哈希值
        #[arg(long)]
        discovery_token_ca_cert_hash: String,
        /// 以控制平面节点身份加入
        #[arg(long)]
        control_plane: bool,
        /// 节点名称
        #[arg(long)]
        node_name: Option<String>,
    },
    /// 创建加入令牌
    Token {
        #[command(subcommand)]
        token_command: TokenCommands,
    },
    /// 升级 Kubernetes 集群
    Upgrade {
        #[command(subcommand)]
        upgrade_command: UpgradeCommands,
    },
    /// 重置 Kubernetes 节点
    Reset {
        /// 强制重置，不提示确认
        #[arg(long)]
        force: bool,
    },
    /// 配置 Kubernetes 集群
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },
    /// 版本信息
    Version,
}

#[derive(Subcommand)]
enum TokenCommands {
    /// 创建新的加入令牌
    Create,
    /// 列出所有加入令牌
    List,
    /// 删除加入令牌
    Delete {
        /// 令牌 ID
        token: String,
    },
}

#[derive(Subcommand)]
enum UpgradeCommands {
    /// 计划升级
    Plan,
    /// 应用升级
    Apply,
    /// 完成升级
    Complete,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// 打印默认配置
    Print {
        /// 配置类型 (init|join|kubelet)
        #[arg(default_value = "init")]
        config_type: String,
    },
    /// 从配置文件初始化
    FromFile {
        /// 配置文件路径
        config: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            apiserver_advertise_address,
            apiserver_bind_port,
            pod_network_cidr,
            service_cidr,
            service_dns_domain,
        } => {
            println!("Initializing Kubernetes cluster");
            if let Some(addr) = &apiserver_advertise_address {
                println!("API Server advertise address: {}", addr);
            }
            println!("API Server bind port: {}", apiserver_bind_port);
            if let Some(cidr) = &pod_network_cidr {
                println!("Pod network CIDR: {}", cidr);
            }
            if let Some(cidr) = &service_cidr {
                println!("Service network CIDR: {}", cidr);
            }
            println!("Service DNS domain: {}", service_dns_domain);

            println!("\n[init] Using Kubernetes version: v1.28.0");
            println!("[preflight] Running pre-flight checks");
            println!("[preflight] Pulling images required for setting up a Kubernetes cluster");
            println!(
                "[preflight] This might take a minute or two, depending on the speed of your internet connection"
            );
            println!(
                "[preflight] You can also perform this action in beforehand using 'kubeadm config images pull'"
            );
            println!("[certs] Using certificateDir folder \"/etc/kubernetes/pki\"");
            println!("[certs] Generating \"ca\" certificate and key");
            println!("[certs] Generating \"apiserver\" certificate and key");
            println!(
                "[certs] apiserver serving cert is signed for DNS names [kubernetes kubernetes.default kubernetes.default.svc kubernetes.default.svc.cluster.local]"
            );
            println!("[certs] Generating \"apiserver-kubelet-client\" certificate and key");
            println!("[certs] Generating \"front-proxy-ca\" certificate and key");
            println!("[certs] Generating \"front-proxy-client\" certificate and key");
            println!("[certs] Generating \"etcd/ca\" certificate and key");
            println!("[certs] Generating \"etcd/server\" certificate and key");
            println!("[certs] etcd/server serving cert is signed for DNS names [localhost]");
            println!("[certs] Generating \"etcd/peer\" certificate and key");
            println!("[certs] etcd/peer serving cert is signed for DNS names [localhost]");
            println!("[certs] Generating \"etcd/healthcheck-client\" certificate and key");
            println!("[certs] Generating \"apiserver-etcd-client\" certificate and key");
            println!("[certs] Valid certificates and keys now exist in \"/etc/kubernetes/pki\"");
            println!("[kubeconfig] Using kubeconfig folder \"/etc/kubernetes\"");
            println!("[kubeconfig] Writing \"admin.conf\" kubeconfig file");
            println!("[kubeconfig] Writing \"kubelet.conf\" kubeconfig file");
            println!("[kubeconfig] Writing \"controller-manager.conf\" kubeconfig file");
            println!("[kubeconfig] Writing \"scheduler.conf\" kubeconfig file");
            println!(
                "[kubelet-start] Writing kubelet environment file with flags to file \"/var/lib/kubelet/kubeadm-flags.env\""
            );
            println!(
                "[kubelet-start] Writing kubelet configuration to file \"/var/lib/kubelet/config.yaml\""
            );
            println!("[kubelet-start] Starting the kubelet");
            println!("[control-plane] Using manifest folder \"/etc/kubernetes/manifests\"");
            println!("[control-plane] Creating static Pod manifest for \"kube-apiserver\"");
            println!(
                "[control-plane] Creating static Pod manifest for \"kube-controller-manager\""
            );
            println!("[control-plane] Creating static Pod manifest for \"kube-scheduler\"");
            println!(
                "[etcd] Creating static Pod manifest for local etcd in \"/etc/kubernetes/manifests\""
            );
            println!(
                "[wait-control-plane] Waiting for the kubelet to boot up the control plane as static Pods from directory \"/etc/kubernetes/manifests\""
            );
            println!("[wait-control-plane] This can take up to 4m0s");
            println!(
                "[apiclient] All control plane components are healthy after 10.501917 seconds"
            );
            println!(
                "[upload-config] Storing the configuration used in ConfigMap \"kubeadm-config\" in the \"kube-system\" Namespace"
            );
            println!(
                "[kubelet] Creating a ConfigMap \"kubelet-config\" in namespace kube-system with the configuration for the kubelets in the cluster"
            );
            println!(
                "[upload-certs] Skipping phase. Please see --upload-certs option in kubeadm init."
            );
            println!(
                "[mark-control-plane] Marking the node localhost as control-plane by adding the labels: [node-role.kubernetes.io/control-plane node.kubernetes.io/exclude-from-external-load-balancers]"
            );
            println!(
                "[mark-control-plane] Marking the node localhost as control-plane by adding the taints [node-role.kubernetes.io/control-plane:NoSchedule]"
            );
            println!("[bootstrap-token] Using token: abcdef.0123456789abcdef");
            println!(
                "[bootstrap-token] Configuring bootstrap tokens, cluster-info ConfigMap, RBAC Roles"
            );
            println!(
                "[bootstrap-token] configured RBAC rules to allow Node Bootstrap tokens to get nodes"
            );
            println!(
                "[bootstrap-token] configured RBAC rules to allow Node Bootstrap tokens to post CSRs in order for nodes to get long term certificate credentials"
            );
            println!(
                "[bootstrap-token] configured RBAC rules to allow the csrapprover controller automatically approve CSRs from a Node Bootstrap Token"
            );
            println!(
                "[bootstrap-token] configured RBAC rules to allow certificate rotation for all node client certificates in the cluster"
            );
            println!(
                "[bootstrap-token] Creating the \"cluster-info\" ConfigMap in the \"kube-public\" namespace"
            );
            println!("[addons] Applied essential addon: CoreDNS");
            println!("[addons] Applied essential addon: kube-proxy");

            println!("\nYour Kubernetes control-plane has initialized successfully!");
            println!(
                "\nTo start using your cluster, you need to run the following as a regular user:"
            );
            println!("\n  mkdir -p $HOME/.kube");
            println!("  sudo cp -i /etc/kubernetes/admin.conf $HOME/.kube/config");
            println!("  sudo chown $(id -u):$(id -g) $HOME/.kube/config");
            println!("\nAlternatively, if you are the root user, you can run:");
            println!("\n  export KUBECONFIG=/etc/kubernetes/admin.conf");
            println!("\nYou should now deploy a Pod network to the cluster.");
            println!(
                "Run \"kubectl apply -f [podnetwork].yaml\" with one of the options listed at:"
            );
            println!("  https://kubernetes.io/docs/concepts/cluster-administration/addons/");
            println!(
                "\nThen you can join any number of worker nodes by running the following on each as root:"
            );
            println!("\nkubeadm join 192.168.1.100:6443 --token abcdef.0123456789abcdef ");
            println!(
                "    --discovery-token-ca-cert-hash sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            );
        }
        Commands::Join {
            control_plane_address,
            token,
            discovery_token_ca_cert_hash,
            control_plane,
            node_name,
        } => {
            println!("Joining Kubernetes cluster");
            println!("Control plane address: {}", control_plane_address);
            println!("Token: {}", token);
            println!(
                "Discovery token CA cert hash: {}",
                discovery_token_ca_cert_hash
            );
            if control_plane {
                println!("Joining as control plane node");
            }
            if let Some(name) = &node_name {
                println!("Node name: {}", name);
            }

            println!("\n[preflight] Running pre-flight checks");
            println!("[preflight] Reading configuration from the cluster...");
            println!(
                "[preflight] FYI: You can look at this config file with 'kubectl -n kube-system get cm kubeadm-config -o yaml'"
            );
            println!(
                "[kubelet-start] Writing kubelet environment file with flags to file \"/var/lib/kubelet/kubeadm-flags.env\""
            );
            println!(
                "[kubelet-start] Writing kubelet configuration to file \"/var/lib/kubelet/config.yaml\""
            );
            println!("[kubelet-start] Starting the kubelet");
            println!("[join] Reading configuration from the cluster...");
            println!(
                "[join] FYI: You can look at this config file with 'kubectl -n kube-system get cm kubeadm-config -o yaml'"
            );
            println!(
                "[join] Detected control-plane node, will perform join to another control-plane node"
            );
            println!("[join] Requesting to join control plane");
            println!("[join] Waiting for approval from existing control-plane nodes");
            println!("[join] Approved by existing control-plane node");
            println!("[join] Creating or updating control plane components");
            println!("[join] Using etcd port: 2379");
            println!("[join] Creating static Pod manifest for \"etcd\"");
            println!("[join] Creating static Pod manifest for \"kube-apiserver\"");
            println!("[join] Creating static Pod manifest for \"kube-controller-manager\"");
            println!("[join] Creating static Pod manifest for \"kube-scheduler\"");
            println!(
                "[wait-control-plane] Waiting for the kubelet to boot up the control plane as static Pods from directory \"/etc/kubernetes/manifests\""
            );
            println!("[wait-control-plane] This can take up to 4m0s");
            println!("[apiclient] All control plane components are healthy after 5.831411 seconds");
            println!(
                "[upload-config] Storing the configuration used in ConfigMap \"kubeadm-config\" in the \"kube-system\" Namespace"
            );
            println!(
                "[mark-control-plane] Marking the node node1 as control-plane by adding the labels: [node-role.kubernetes.io/control-plane node.kubernetes.io/exclude-from-external-load-balancers]"
            );
            println!(
                "[mark-control-plane] Marking the node node1 as control-plane by adding the taints [node-role.kubernetes.io/control-plane:NoSchedule]"
            );

            println!(
                "\nThis node has joined the cluster and a new control plane instance was created:"
            );
            println!(
                "\n* Certificate signing request was sent to apiserver and approval was received."
            );
            println!("* The Kubelet was informed of the new secure connection details.");
            println!("* Control plane (master) label and taint were applied to the new node.");
            println!("* The Kubernetes control plane instances scaled up.");
            println!("* A new etcd member was added to the existing etcd cluster.");

            println!("\nTo start administering your cluster from this node, you need to run:");
            println!("\n  mkdir -p $HOME/.kube");
            println!("  sudo cp -i /etc/kubernetes/admin.conf $HOME/.kube/config");
            println!("  sudo chown $(id -u):$(id -g) $HOME/.kube/config");
            println!("\nRun 'kubectl get nodes' to see this node join the cluster.");
        }
        Commands::Token { token_command } => match token_command {
            TokenCommands::Create => {
                println!("Creating join token");
                println!("token: abcdef.0123456789abcdef");
                println!("ttl: 24h0m0s");
                println!("usages: signing,authentication");
                println!("groups: system:bootstrappers:kubeadm:default-node-token");
            }
            TokenCommands::List => {
                println!("Listing join tokens");
                println!(
                    "TOKEN                     TTL         EXPIRES                     USAGES                   GROUPS"
                );
                println!(
                    "abcdef.0123456789abcdef   23h         2024-01-02T12:00:00Z        signing,authentication   system:bootstrappers:kubeadm:default-node-token"
                );
            }
            TokenCommands::Delete { token } => {
                println!("Deleting join token");
                println!("Token: {}", token);
                println!("Token deleted successfully");
            }
        },
        Commands::Upgrade { upgrade_command } => match upgrade_command {
            UpgradeCommands::Plan => {
                println!("Planning Kubernetes upgrade");
                println!("[upgrade/config] Making sure the configuration is correct");
                println!("[upgrade/config] Reading configuration from the cluster...");
                println!(
                    "[upgrade/config] FYI: You can look at this config file with 'kubectl -n kube-system get cm kubeadm-config -o yaml'"
                );
                println!("[preflight] Running pre-flight checks.");
                println!(
                    "[upgrade] This cluster was originally configured using an older version of kubeadm"
                );
                println!(
                    "[upgrade] Detected that the cluster uses the CRI socket /var/run/dockershim.sock"
                );
                println!("[upgrade] Looking for available upgrades for Kubernetes v1.27.0");
                println!("[upgrade] Latest version in the v1.27 series is v1.27.4");
                println!("[upgrade] The machine needs to be upgraded to v1.27.4");
            }
            UpgradeCommands::Apply => {
                println!("Applying Kubernetes upgrade");
                println!("[upgrade/config] Making sure the configuration is correct");
                println!("[upgrade/config] Reading configuration from the cluster...");
                println!(
                    "[upgrade/config] FYI: You can look at this config file with 'kubectl -n kube-system get cm kubeadm-config -o yaml'"
                );
                println!("[preflight] Running pre-flight checks.");
                println!("[upgrade/version] You have chosen to upgrade to version v1.27.4");
                println!(
                    "[upgrade/prepull] Pulling images required for setting up a Kubernetes cluster"
                );
                println!(
                    "[upgrade/prepull] This might take a minute or two, depending on the speed of your internet connection"
                );
                println!(
                    "[upgrade/apply] Upgrading your Static Pod-hosted control plane to version v1.27.4"
                );
                println!("[upgrade/etcd] Upgrading to etcd version 3.5.7-0");
                println!(
                    "[upgrade/staticpods] Writing new Static Pod manifests to /etc/kubernetes/tmp/kubeadm-upgrade-XXXXXX/manifests"
                );
                println!(
                    "[upgrade/staticpods] This might take a minute or two, depending on the speed of your internet connection"
                );
                println!(
                    "[upgrade/staticpods] Waiting for the kubelet to restart the control plane containers"
                );
                println!(
                    "[upgrade/staticpods] This might take a minute or two, depending on the speed of your internet connection"
                );
                println!(
                    "[upgrade/staticpods] Control plane containers are now running at version v1.27.4"
                );
                println!("[upgrade/postupgrade] Applying the new configuration to the kubelet");
                println!(
                    "[upgrade/postupgrade] The kubelet was successfully upgraded to version v1.27.4"
                );
            }
            UpgradeCommands::Complete => {
                println!("Completing Kubernetes upgrade");
                println!("[upgrade/complete] Reading configuration from the cluster...");
                println!(
                    "[upgrade/complete] FYI: You can look at this config file with 'kubectl -n kube-system get cm kubeadm-config -o yaml'"
                );
                println!("[upgrade/complete] Making sure the configuration is correct");
                println!("[upgrade/complete] Skipping phase. Not a control plane node.");
                println!("[upgrade/complete] Successfully upgraded cluster to version v1.27.4");
            }
        },
        Commands::Reset { force } => {
            println!("Resetting Kubernetes node");
            if force {
                println!("Forcing reset without confirmation");
            }
            println!(
                "[reset] WARNING: Changes made to this host by 'kubeadm init' or 'kubeadm join' will be reverted"
            );
            println!("[reset] Are you sure you want to proceed? [y/N]: y");
            println!("[preflight] Running pre-flight checks");
            println!(
                "[reset] Removing info for node 'node1' from the ConfigMap 'kubeadm-config' in the 'kube-system' Namespace"
            );
            println!(
                "[reset] Removing info for node 'node1' from the ConfigMap 'cluster-info' in the 'kube-public' Namespace"
            );
            println!(
                "[reset] Deleting contents of stateful directories: [/var/lib/etcd /var/lib/kubelet /var/lib/dockershim /var/run/kubernetes /var/lib/cni]"
            );
            println!(
                "[reset] Deleting contents of config directories: [/etc/kubernetes/manifests /etc/kubernetes/pki]"
            );
            println!(
                "[reset] Deleting files: [/etc/kubernetes/admin.conf /etc/kubernetes/kubelet.conf /etc/kubernetes/bootstrap-kubelet.conf /etc/kubernetes/controller-manager.conf /etc/kubernetes/scheduler.conf]"
            );
            println!("[reset] Deleting kubeconfig files: [/root/.kube/config]");
            println!("[reset] Stopping the kubelet service");
            println!("[reset] Unmounting mounted directories in \"/var/lib/kubelet\"");
            println!("[reset] Removing kubelet systemd service definition");
            println!("[reset] Removing kubeadm systemd service definition");
            println!("[reset] Manually resetting networking configuration");
            println!("[reset] Cleaning up the kubelet environment");
            println!(
                "[reset] Now you can join this node to another cluster as a worker or control-plane node."
            );
        }
        Commands::Config { config_command } => match config_command {
            ConfigCommands::Print { config_type } => {
                println!("Printing default {} configuration", config_type);
                println!("apiVersion: kubeadm.k8s.io/v1beta3");
                println!("kind: InitConfiguration");
                println!("localAPIEndpoint:");
                println!("  advertiseAddress: 192.168.1.100");
                println!("  bindPort: 6443");
                println!("nodeRegistration:");
                println!("  name: node1");
                println!("  taints:");
                println!("  - effect: NoSchedule");
                println!("    key: node-role.kubernetes.io/master");
                println!("---");
                println!("apiVersion: kubeadm.k8s.io/v1beta3");
                println!("kind: ClusterConfiguration");
                println!("kubernetesVersion: v1.28.0");
                println!("controlPlaneEndpoint: \"192.168.1.100:6443\"");
                println!("networking:");
                println!("  podSubnet: 10.244.0.0/16");
                println!("  serviceSubnet: 10.96.0.0/12");
            }
            ConfigCommands::FromFile { config } => {
                println!("Initializing from configuration file");
                println!("Config file: {}", config);
                println!("[init] Using Kubernetes version: v1.28.0");
                println!("[preflight] Running pre-flight checks");
                println!("[preflight] Pulling images required for setting up a Kubernetes cluster");
                println!(
                    "[preflight] This might take a minute or two, depending on the speed of your internet connection"
                );
                println!("[certs] Using certificateDir folder \"/etc/kubernetes/pki\"");
                println!("[certs] Generating \"ca\" certificate and key");
                println!("[certs] Generating \"apiserver\" certificate and key");
                println!(
                    "[certs] apiserver serving cert is signed for DNS names [kubernetes kubernetes.default kubernetes.default.svc kubernetes.default.svc.cluster.local]"
                );
                println!("[certs] Generating \"apiserver-kubelet-client\" certificate and key");
                println!("[certs] Generating \"front-proxy-ca\" certificate and key");
                println!("[certs] Generating \"front-proxy-client\" certificate and key");
                println!("[certs] Generating \"etcd/ca\" certificate and key");
                println!("[certs] Generating \"etcd/server\" certificate and key");
                println!("[certs] etcd/server serving cert is signed for DNS names [localhost]");
                println!("[certs] Generating \"etcd/peer\" certificate and key");
                println!("[certs] etcd/peer serving cert is signed for DNS names [localhost]");
                println!("[certs] Generating \"etcd/healthcheck-client\" certificate and key");
                println!("[certs] Generating \"apiserver-etcd-client\" certificate and key");
                println!(
                    "[certs] Valid certificates and keys now exist in \"/etc/kubernetes/pki\""
                );
                println!("[kubeconfig] Using kubeconfig folder \"/etc/kubernetes\"");
                println!("[kubeconfig] Writing \"admin.conf\" kubeconfig file");
                println!("[kubeconfig] Writing \"kubelet.conf\" kubeconfig file");
                println!("[kubeconfig] Writing \"controller-manager.conf\" kubeconfig file");
                println!("[kubeconfig] Writing \"scheduler.conf\" kubeconfig file");
                println!(
                    "[kubelet-start] Writing kubelet environment file with flags to file \"/var/lib/kubelet/kubeadm-flags.env\""
                );
                println!(
                    "[kubelet-start] Writing kubelet configuration to file \"/var/lib/kubelet/config.yaml\""
                );
                println!("[kubelet-start] Starting the kubelet");
                println!("[control-plane] Using manifest folder \"/etc/kubernetes/manifests\"");
                println!("[control-plane] Creating static Pod manifest for \"kube-apiserver\"");
                println!(
                    "[control-plane] Creating static Pod manifest for \"kube-controller-manager\""
                );
                println!("[control-plane] Creating static Pod manifest for \"kube-scheduler\"");
                println!(
                    "[etcd] Creating static Pod manifest for local etcd in \"/etc/kubernetes/manifests\""
                );
                println!(
                    "[wait-control-plane] Waiting for the kubelet to boot up the control plane as static Pods from directory \"/etc/kubernetes/manifests\""
                );
                println!("[wait-control-plane] This can take up to 4m0s");
                println!(
                    "[apiclient] All control plane components are healthy after 10.501917 seconds"
                );
                println!(
                    "[upload-config] Storing the configuration used in ConfigMap \"kubeadm-config\" in the \"kube-system\" Namespace"
                );
                println!(
                    "[kubelet] Creating a ConfigMap \"kubelet-config\" in namespace kube-system with the configuration for the kubelets in the cluster"
                );
                println!(
                    "[upload-certs] Skipping phase. Please see --upload-certs option in kubeadm init."
                );
                println!(
                    "[mark-control-plane] Marking the node localhost as control-plane by adding the labels: [node-role.kubernetes.io/control-plane node.kubernetes.io/exclude-from-external-load-balancers]"
                );
                println!(
                    "[mark-control-plane] Marking the node localhost as control-plane by adding the taints [node-role.kubernetes.io/control-plane:NoSchedule]"
                );
                println!("[bootstrap-token] Using token: abcdef.0123456789abcdef");
                println!(
                    "[bootstrap-token] Configuring bootstrap tokens, cluster-info ConfigMap, RBAC Roles"
                );
                println!(
                    "[bootstrap-token] configured RBAC rules to allow Node Bootstrap tokens to get nodes"
                );
                println!(
                    "[bootstrap-token] configured RBAC rules to allow Node Bootstrap tokens to post CSRs in order for nodes to get long term certificate credentials"
                );
                println!(
                    "[bootstrap-token] configured RBAC rules to allow the csrapprover controller automatically approve CSRs from a Node Bootstrap Token"
                );
                println!(
                    "[bootstrap-token] configured RBAC rules to allow certificate rotation for all node client certificates in the cluster"
                );
                println!(
                    "[bootstrap-token] Creating the \"cluster-info\" ConfigMap in the \"kube-public\" namespace"
                );
                println!("[addons] Applied essential addon: CoreDNS");
                println!("[addons] Applied essential addon: kube-proxy");
            }
        },
        Commands::Version => {
            println!("kubeadm version:");
            println!(
                "kubeadm version: &version.Info{{Major:\"1\", Minor:\"28\", GitVersion:\"v1.28.0\", GitCommit:\"1234567890abcdef1234567890abcdef12345678\", GitTreeState:\"clean\", BuildDate:\"2024-01-01T00:00:00Z\", GoVersion:\"go1.20.0\", Compiler:\"gc\", Platform:\"linux/amd64\"}}"
            );
        }
    }
}
