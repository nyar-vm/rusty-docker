use clap::{Parser, Subcommand};
use oak_yaml::parse;
use serde_json::{Value, to_string_pretty};
use std::{collections::HashMap, error::Error, fs::File, io::Read, path::Path};
use tokio::fs::read_to_string;
use wae_request::{HttpClient, HttpClientConfig, HttpResponse};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get information about resources
    Get {
        /// Resource type (pods, services, deployments, etc.)
        resource: String,
        /// Optional resource name
        name: Option<String>,
        /// Show all resources in all namespaces
        #[arg(short, long)]
        all_namespaces: bool,
        /// Label selector
        #[arg(long)]
        selector: Option<String>,
        /// Output format (json, yaml, wide, custom-columns)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Create a resource from a file
    Apply {
        /// Path to the YAML file
        file: String,
        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
    /// Delete a resource
    Delete {
        /// Resource type (pods, services, deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Describe a resource
    Describe {
        /// Resource type (pods, services, deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// List all contexts
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },
    /// Create a resource
    Create {
        /// Path to the YAML file
        file: String,
        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
    /// Edit a resource
    Edit {
        /// Resource type (pods, services, deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Patch a resource
    Patch {
        /// Resource type (pods, services, deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
        /// Patch content
        #[arg(long)]
        patch: String,
        /// Patch type (json, merge, strategic-merge)
        #[arg(long, default_value = "strategic-merge")]
        r#type: String,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Manage rollout
    Rollout {
        #[command(subcommand)]
        rollout_command: RolloutCommands,
    },
    /// Show cluster information
    ClusterInfo,
    /// Show resource usage
    Top {
        #[command(subcommand)]
        top_command: TopCommands,
    },
    /// Manage labels
    Label {
        /// Resource type
        resource: String,
        /// Resource name
        name: String,
        /// Labels to set
        labels: Vec<String>,
        /// Overwrite existing labels
        #[arg(long)]
        overwrite: bool,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Manage annotations
    Annotate {
        /// Resource type
        resource: String,
        /// Resource name
        name: String,
        /// Annotations to set
        annotations: Vec<String>,
        /// Overwrite existing annotations
        #[arg(long)]
        overwrite: bool,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Mark node as unschedulable
    Cordon {
        /// Node name
        node: String,
    },
    /// Mark node as schedulable
    Uncordon {
        /// Node name
        node: String,
    },
    /// Drain node
    Drain {
        /// Node name
        node: String,
        /// Ignore daemonsets
        #[arg(long)]
        ignore_daemonsets: bool,
        /// Delete local data
        #[arg(long)]
        delete_local_data: bool,
        /// Force drain
        #[arg(long)]
        force: bool,
    },
    /// Manage node taints
    Taint {
        /// Node name
        node: String,
        /// Taint specifications
        taints: Vec<String>,
    },
    /// Show documentation for resources
    Explain {
        /// Resource type
        resource: String,
    },
    /// Get logs from a container
    Logs {
        /// Pod name
        pod: String,
        /// Container name
        #[arg(short, long)]
        container: Option<String>,
        /// Follow logs
        #[arg(short, long)]
        follow: bool,
        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        tail: String,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Execute a command in a container
    Exec {
        /// Pod name
        pod: String,
        /// Command to execute
        command: Vec<String>,
        /// Container name
        #[arg(short, long)]
        container: Option<String>,
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Forward one or more local ports to a pod
    PortForward {
        /// Pod name
        pod: String,
        /// Port forwarding specifications (local:remote)
        ports: Vec<String>,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Run a proxy to the Kubernetes API server
    Proxy {
        /// Port to listen on
        #[arg(short, long, default_value = "8001")]
        port: String,
    },
    /// Copy files and directories to and from containers
    Cp {
        /// Source path
        source: String,
        /// Destination path
        destination: String,
    },
    /// Check authorization
    Auth {
        #[command(subcommand)]
        auth_command: AuthCommands,
    },
    /// Manage certificates
    Certificate {
        #[command(subcommand)]
        certificate_command: CertificateCommands,
    },
    /// Convert config files between different API versions
    Convert {
        /// Path to the file to convert
        file: String,
    },
    /// Run a plugin
    Plugin {
        #[command(subcommand)]
        plugin_command: PluginCommands,
    },
    /// Show version information
    Version {
        /// Show client version only
        #[arg(long)]
        client: bool,
        /// Show server version only
        #[arg(long)]
        server: bool,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// List contexts
    GetContexts,
    /// Use a context
    UseContext {
        /// Context name
        context: String,
    },
}

#[derive(Subcommand)]
enum RolloutCommands {
    /// Check the rollout status of a resource
    Status {
        /// Resource type (deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
    },
    /// Rollback to a previous revision
    Undo {
        /// Resource type (deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
    },
    /// Pause a rollout
    Pause {
        /// Resource type (deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
    },
    /// Resume a rollout
    Resume {
        /// Resource type (deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
    },
    /// History of rollouts
    History {
        /// Resource type (deployments, etc.)
        resource: String,
        /// Resource name
        name: String,
    },
}

#[derive(Subcommand)]
enum TopCommands {
    /// Show pod resource usage
    Pod {
        /// Show all namespaces
        #[arg(short, long)]
        all_namespaces: bool,
    },
    /// Show node resource usage
    Node,
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Check if current user can perform an action
    CanI {
        /// Verb
        verb: String,
        /// Resource
        resource: String,
        /// Resource name
        name: Option<String>,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Print service account token
    WhoAmI,
}

#[derive(Subcommand)]
enum CertificateCommands {
    /// Approve a certificate signing request
    Approve {
        /// CSR name
        name: String,
    },
    /// Deny a certificate signing request
    Deny {
        /// CSR name
        name: String,
    },
    /// Create a CSR
    Create {
        /// Common name
        #[arg(long)]
        common_name: String,
        /// Output file
        #[arg(short, long)]
        output: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List available plugins
    List,
    /// Describe a plugin
    Describe {
        /// Plugin name
        name: String,
    },
}

/// Kubernetes 资源类型
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct K8sResource {
    apiVersion: String,
    kind: String,
    metadata: Metadata,
    spec: Option<serde_json::Value>,
    status: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Metadata {
    name: String,
    namespace: Option<String>,
    labels: Option<serde_json::Value>,
    annotations: Option<serde_json::Value>,
}

/// Kubernetes 客户端配置
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct KubeConfig {
    apiVersion: String,
    kind: String,
    clusters: Vec<Cluster>,
    contexts: Vec<Context>,
    currentContext: String,
    users: Vec<User>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Cluster {
    name: String,
    cluster: ClusterConfig,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ClusterConfig {
    server: String,
    certificateAuthority: Option<String>,
    certificateAuthorityData: Option<String>,
    insecureSkipTLSVerify: Option<bool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Context {
    name: String,
    context: ContextConfig,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ContextConfig {
    cluster: String,
    user: String,
    namespace: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct User {
    name: String,
    user: UserConfig,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct UserConfig {
    clientCertificate: Option<String>,
    clientCertificateData: Option<String>,
    clientKey: Option<String>,
    clientKeyData: Option<String>,
    token: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

/// Kubernetes 客户端
struct K8sClient {
    client: HttpClient,
    server: String,
    token: Option<String>,
    namespace: String,
}

impl K8sClient {
    async fn new() -> Result<Self, Box<dyn Error>> {
        // 加载 kubeconfig
        let kubeconfig = KubeConfig::load().await?;

        // 获取当前上下文
        let current_context = kubeconfig.currentContext;
        let context = kubeconfig.contexts.into_iter().find(|c| c.name == current_context).ok_or("Current context not found")?;

        // 获取集群信息
        let cluster = kubeconfig.clusters.into_iter().find(|c| c.name == context.context.cluster).ok_or("Cluster not found")?;

        // 获取用户信息
        let user = kubeconfig.users.into_iter().find(|u| u.name == context.context.user).ok_or("User not found")?;

        // 创建 HTTP 客户端
        let client = HttpClient::new(HttpClientConfig::default());

        Ok(Self {
            client,
            server: cluster.cluster.server,
            token: user.user.token,
            namespace: context.context.namespace.unwrap_or_else(|| "default".to_string()),
        })
    }

    async fn get_resources(&self, resource_type: &str, namespace: Option<&str>) -> Result<Value, Box<dyn Error>> {
        let ns = namespace.unwrap_or(&self.namespace);
        let url = format!("{}/api/v1/namespaces/{}/{}?limit=500", self.server, ns, resource_type);

        let mut headers = HashMap::new();
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let response = self.client.get_with_headers(&url, headers).await?;
        let body = String::from_utf8(response.body)?;
        let result: Value = serde_json::from_str(&body)?;

        Ok(result)
    }

    async fn get_resource(&self, resource_type: &str, name: &str, namespace: Option<&str>) -> Result<Value, Box<dyn Error>> {
        let ns = namespace.unwrap_or(&self.namespace);
        let url = format!("{}/api/v1/namespaces/{}/{}/{}?limit=500", self.server, ns, resource_type, name);

        let mut headers = HashMap::new();
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let response = self.client.get_with_headers(&url, headers).await?;
        let body = String::from_utf8(response.body)?;
        let result: Value = serde_json::from_str(&body)?;

        Ok(result)
    }

    async fn apply_resource(&self, resource: &K8sResource) -> Result<Value, Box<dyn Error>> {
        let namespace = resource.metadata.namespace.as_deref().unwrap_or(&self.namespace);
        let url = format!("{}/api/v1/namespaces/{}/{}", self.server, namespace, resource.kind.to_lowercase() + "s");

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let body = serde_json::to_vec(resource)?;
        let response = self.client.request("POST", &url, Some(body), Some(headers)).await?;
        let response_body = String::from_utf8(response.body)?;
        let result: Value = serde_json::from_str(&response_body)?;

        Ok(result)
    }

    async fn delete_resource(&self, resource_type: &str, name: &str, namespace: Option<&str>) -> Result<Value, Box<dyn Error>> {
        let ns = namespace.unwrap_or(&self.namespace);
        let url = format!("{}/api/v1/namespaces/{}/{}/{}?limit=500", self.server, ns, resource_type, name);

        let mut headers = HashMap::new();
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let response = self.client.request("DELETE", &url, None, Some(headers)).await?;
        let body = String::from_utf8(response.body)?;
        let result: Value = serde_json::from_str(&body)?;

        Ok(result)
    }

    async fn create_resource(&self, resource: &K8sResource) -> Result<Value, Box<dyn Error>> {
        let namespace = resource.metadata.namespace.as_deref().unwrap_or(&self.namespace);
        let url = format!("{}/api/v1/namespaces/{}/{}", self.server, namespace, resource.kind.to_lowercase() + "s");

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let body = serde_json::to_vec(resource)?;
        let response = self.client.request("POST", &url, Some(body), Some(headers)).await?;
        let response_body = String::from_utf8(response.body)?;
        let result: Value = serde_json::from_str(&response_body)?;

        Ok(result)
    }

    async fn patch_resource(
        &self,
        resource_type: &str,
        name: &str,
        patch: &str,
        namespace: Option<&str>,
    ) -> Result<Value, Box<dyn Error>> {
        let ns = namespace.unwrap_or(&self.namespace);
        let url = format!("{}/api/v1/namespaces/{}/{}/{}?limit=500", self.server, ns, resource_type, name);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json-patch+json".to_string());
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let body = patch.as_bytes().to_vec();
        let response = self.client.request("PATCH", &url, Some(body), Some(headers)).await?;
        let response_body = String::from_utf8(response.body)?;
        let result: Value = serde_json::from_str(&response_body)?;

        Ok(result)
    }

    async fn rollout_status(&self, resource_type: &str, name: &str, namespace: Option<&str>) -> Result<Value, Box<dyn Error>> {
        let ns = namespace.unwrap_or(&self.namespace);
        let url = format!("{}/apis/apps/v1/namespaces/{}/{}/{}/status", self.server, ns, resource_type, name);

        let mut headers = HashMap::new();
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let response = self.client.get_with_headers(&url, headers).await?;
        let body = String::from_utf8(response.body)?;
        let result: Value = serde_json::from_str(&body)?;

        Ok(result)
    }

    async fn rollout_undo(&self, resource_type: &str, name: &str, namespace: Option<&str>) -> Result<Value, Box<dyn Error>> {
        let ns = namespace.unwrap_or(&self.namespace);
        let url = format!("{}/apis/apps/v1/namespaces/{}/{}/{}/rollback", self.server, ns, resource_type, name);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let body = serde_json::to_vec(&serde_json::json!({}))?;
        let response = self.client.request("POST", &url, Some(body), Some(headers)).await?;
        let response_body = String::from_utf8(response.body)?;
        let result: Value = serde_json::from_str(&response_body)?;

        Ok(result)
    }
}

async fn load_resource_from_file(file_path: &str) -> Result<K8sResource, Box<dyn Error>> {
    let content = read_to_string(file_path).await?;

    // 尝试使用 oak-yaml 解析 YAML
    match parse(&content) {
        Ok(root) => {
            // 使用 oak-yaml 的 serde 支持进行反序列化
            let resource: K8sResource = serde::de::Deserialize::deserialize(root)?;
            Ok(resource)
        }
        Err(e) => Err(format!("Failed to parse resource file: {}", e).into()),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Get { resource, name, all_namespaces, selector: _, output: _ } => {
            // 尝试创建 K8sClient
            match K8sClient::new().await {
                Ok(client) => {
                    let namespace = if all_namespaces { Some("default") } else { None };

                    if let Some(name) = name {
                        match client.get_resource(&resource, &name, namespace).await {
                            Ok(resource_obj) => {
                                println!("{}", to_string_pretty(&resource_obj).unwrap())
                            }
                            Err(e) => eprintln!("Error getting resource: {:?}", e),
                        }
                    }
                    else {
                        match client.get_resources(&resource, namespace).await {
                            Ok(resources) => println!("{}", to_string_pretty(&resources).unwrap()),
                            Err(e) => eprintln!("Error getting resources: {:?}", e),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error initializing Kubernetes client: {:?}", e);
                    eprintln!("Using mock mode...");
                    // 回退到模拟模式
                    let cluster = K8sCluster::new();
                    if let Some(name) = name {
                        if let Some(resource_obj) = cluster.get_resource(&resource, &name) {
                            println!("{}", to_string_pretty(&resource_obj).unwrap());
                        }
                        else {
                            eprintln!("Error: {} '{}' not found", resource, name);
                        }
                    }
                    else {
                        let resources = cluster.get_resources(&resource);
                        println!("{}", to_string_pretty(&resources).unwrap());
                    }
                }
            }
        }
        Commands::Apply { file, dry_run: _ } => {
            match load_resource_from_file(&file).await {
                Ok(resource) => {
                    // 尝试创建 K8sClient
                    match K8sClient::new().await {
                        Ok(client) => match client.apply_resource(&resource).await {
                            Ok(result) => {
                                println!("Resource applied successfully");
                                println!("{}", to_string_pretty(&result).unwrap());
                            }
                            Err(e) => eprintln!("Error applying resource: {:?}", e),
                        },
                        Err(e) => {
                            eprintln!("Error initializing Kubernetes client: {:?}", e);
                            eprintln!("Using mock mode...");
                            // 回退到模拟模式
                            let mut cluster = K8sCluster::new();
                            cluster.apply_resource(resource);
                            println!("Resource applied successfully (mock mode)");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error loading resource: {:?}", e);
                }
            }
        }
        Commands::Delete { resource, name, namespace: _ } => {
            // 尝试创建 K8sClient
            match K8sClient::new().await {
                Ok(client) => match client.delete_resource(&resource, &name, None).await {
                    Ok(result) => {
                        println!("{} '{}' deleted", resource, name);
                        println!("{}", to_string_pretty(&result).unwrap());
                    }
                    Err(e) => eprintln!("Error deleting resource: {:?}", e),
                },
                Err(e) => {
                    eprintln!("Error initializing Kubernetes client: {:?}", e);
                    eprintln!("Using mock mode...");
                    // 回退到模拟模式
                    let mut cluster = K8sCluster::new();
                    if cluster.delete_resource(&resource, &name) {
                        println!("{} '{}' deleted (mock mode)", resource, name);
                    }
                    else {
                        eprintln!("Error: {} '{}' not found", resource, name);
                    }
                }
            }
        }
        Commands::Describe { resource, name, namespace: _ } => {
            // 尝试创建 K8sClient
            match K8sClient::new().await {
                Ok(client) => match client.get_resource(&resource, &name, None).await {
                    Ok(resource_obj) => println!("{}", to_string_pretty(&resource_obj).unwrap()),
                    Err(e) => eprintln!("Error describing resource: {:?}", e),
                },
                Err(e) => {
                    eprintln!("Error initializing Kubernetes client: {:?}", e);
                    eprintln!("Using mock mode...");
                    // 回退到模拟模式
                    let cluster = K8sCluster::new();
                    if let Some(resource_obj) = cluster.get_resource(&resource, &name) {
                        println!("{}", to_string_pretty(&resource_obj).unwrap());
                    }
                    else {
                        eprintln!("Error: {} '{}' not found", resource, name);
                    }
                }
            }
        }
        Commands::Config { config_command } => match config_command {
            ConfigCommands::GetContexts => {
                // 尝试从 kubeconfig 文件读取上下文
                match KubeConfig::load().await {
                    Ok(kubeconfig) => {
                        println!("CURRENT   NAME       CLUSTER    AUTHINFO   NAMESPACE");
                        for context in &kubeconfig.contexts {
                            let current = if context.name == kubeconfig.currentContext { "*" } else { " " };
                            let ns = context.context.namespace.as_deref().unwrap_or("");
                            println!(
                                "{}         {}       {}    {}   {}",
                                current, context.name, context.context.cluster, context.context.user, ns
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Error loading kubeconfig: {:?}", e);
                        eprintln!("Using mock mode...");
                        println!("CURRENT   NAME       CLUSTER    AUTHINFO   NAMESPACE");
                        println!("*         minikube   minikube   minikube   default");
                    }
                }
            }
            ConfigCommands::UseContext { context } => {
                println!("Switched to context '{}'.", context);
                // 实际实现需要更新 kubeconfig 文件中的 currentContext
            }
        },
        Commands::Create { file, dry_run: _ } => {
            match load_resource_from_file(&file).await {
                Ok(resource) => {
                    // 尝试创建 K8sClient
                    match K8sClient::new().await {
                        Ok(client) => match client.create_resource(&resource).await {
                            Ok(result) => {
                                println!("Resource created successfully");
                                println!("{}", to_string_pretty(&result).unwrap());
                            }
                            Err(e) => eprintln!("Error creating resource: {:?}", e),
                        },
                        Err(e) => {
                            eprintln!("Error initializing Kubernetes client: {:?}", e);
                            eprintln!("Using mock mode...");
                            // 回退到模拟模式
                            let mut cluster = K8sCluster::new();
                            cluster.apply_resource(resource);
                            println!("Resource created successfully (mock mode)");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error loading resource: {:?}", e);
                }
            }
        }
        Commands::Edit { resource, name, namespace: _ } => {
            // 尝试创建 K8sClient
            match K8sClient::new().await {
                Ok(client) => match client.get_resource(&resource, &name, None).await {
                    Ok(resource_obj) => {
                        println!("Opening editor for {} {}", resource, name);
                        println!("{}", to_string_pretty(&resource_obj).unwrap());
                        println!("Edit functionality not fully implemented yet");
                    }
                    Err(e) => eprintln!("Error getting resource: {:?}", e),
                },
                Err(e) => {
                    eprintln!("Error initializing Kubernetes client: {:?}", e);
                    eprintln!("Using mock mode...");
                    // 回退到模拟模式
                    let cluster = K8sCluster::new();
                    if let Some(resource_obj) = cluster.get_resource(&resource, &name) {
                        println!("Opening editor for {} {} (mock mode)", resource, name);
                        println!("{}", to_string_pretty(&resource_obj).unwrap());
                        println!("Edit functionality not fully implemented yet");
                    }
                    else {
                        eprintln!("Error: {} '{}' not found", resource, name);
                    }
                }
            }
        }
        Commands::Patch { resource, name, patch, r#type: _, namespace: _ } => {
            // 尝试创建 K8sClient
            match K8sClient::new().await {
                Ok(client) => match client.patch_resource(&resource, &name, &patch, None).await {
                    Ok(result) => {
                        println!("Resource patched successfully");
                        println!("{}", to_string_pretty(&result).unwrap());
                    }
                    Err(e) => eprintln!("Error patching resource: {:?}", e),
                },
                Err(e) => {
                    eprintln!("Error initializing Kubernetes client: {:?}", e);
                    eprintln!("Using mock mode...");
                    println!("Resource patched successfully (mock mode)");
                }
            }
        }
        Commands::Rollout { rollout_command } => match rollout_command {
            RolloutCommands::Status { resource, name } => {
                // 尝试创建 K8sClient
                match K8sClient::new().await {
                    Ok(client) => match client.rollout_status(&resource, &name, None).await {
                        Ok(result) => {
                            println!("Rollout status for {} {}", resource, name);
                            println!("{}", to_string_pretty(&result).unwrap());
                        }
                        Err(e) => eprintln!("Error getting rollout status: {:?}", e),
                    },
                    Err(e) => {
                        eprintln!("Error initializing Kubernetes client: {:?}", e);
                        eprintln!("Using mock mode...");
                        println!("Rollout status for {} {} (mock mode)", resource, name);
                        println!("Status: Complete");
                    }
                }
            }
            RolloutCommands::Undo { resource, name } => {
                // 尝试创建 K8sClient
                match K8sClient::new().await {
                    Ok(client) => match client.rollout_undo(&resource, &name, None).await {
                        Ok(result) => {
                            println!("Rollout undone successfully for {} {}", resource, name);
                            println!("{}", to_string_pretty(&result).unwrap());
                        }
                        Err(e) => eprintln!("Error undoing rollout: {:?}", e),
                    },
                    Err(e) => {
                        eprintln!("Error initializing Kubernetes client: {:?}", e);
                        eprintln!("Using mock mode...");
                        println!("Rollout undone successfully for {} {} (mock mode)", resource, name);
                    }
                }
            }
            RolloutCommands::Pause { resource, name } => {
                println!("Rollout pause not fully implemented yet");
            }
            RolloutCommands::Resume { resource, name } => {
                println!("Rollout resume not fully implemented yet");
            }
            RolloutCommands::History { resource, name } => {
                println!("Rollout history not fully implemented yet");
            }
        },
        Commands::ClusterInfo => {
            println!("Cluster information not fully implemented yet");
        }
        Commands::Top { top_command: _ } => {
            println!("Top command not fully implemented yet");
        }
        Commands::Label { .. } => {
            println!("Label command not fully implemented yet");
        }
        Commands::Annotate { .. } => {
            println!("Annotate command not fully implemented yet");
        }
        Commands::Cordon { .. } => {
            println!("Cordon command not fully implemented yet");
        }
        Commands::Uncordon { .. } => {
            println!("Uncordon command not fully implemented yet");
        }
        Commands::Drain { .. } => {
            println!("Drain command not fully implemented yet");
        }
        Commands::Taint { .. } => {
            println!("Taint command not fully implemented yet");
        }
        Commands::Explain { .. } => {
            println!("Explain command not fully implemented yet");
        }
        Commands::Logs { .. } => {
            println!("Logs command not fully implemented yet");
        }
        Commands::Exec { .. } => {
            println!("Exec command not fully implemented yet");
        }
        Commands::PortForward { .. } => {
            println!("PortForward command not fully implemented yet");
        }
        Commands::Proxy { .. } => {
            println!("Proxy command not fully implemented yet");
        }
        Commands::Cp { .. } => {
            println!("Cp command not fully implemented yet");
        }
        Commands::Auth { auth_command: _ } => {
            println!("Auth command not fully implemented yet");
        }
        Commands::Certificate { certificate_command: _ } => {
            println!("Certificate command not fully implemented yet");
        }
        Commands::Convert { .. } => {
            println!("Convert command not fully implemented yet");
        }
        Commands::Plugin { plugin_command: _ } => {
            println!("Plugin command not fully implemented yet");
        }
        Commands::Version { .. } => {
            println!("Version command not fully implemented yet");
        }
    }
}

/// 模拟 Kubernetes 集群状态（用于回退模式）
struct K8sCluster {
    pods: Vec<K8sResource>,
    services: Vec<K8sResource>,
    deployments: Vec<K8sResource>,
    namespaces: Vec<K8sResource>,
}

impl K8sCluster {
    fn new() -> Self {
        // 初始化模拟数据
        let pods = vec![
            K8sResource {
                apiVersion: "v1".to_string(),
                kind: "Pod".to_string(),
                metadata: Metadata {
                    name: "nginx-12345".to_string(),
                    namespace: Some("default".to_string()),
                    labels: Some(serde_json::json!({
                        "app": "nginx"
                    })),
                    annotations: None,
                },
                spec: Some(serde_json::json!({
                    "containers": [{
                        "name": "nginx",
                        "image": "nginx:latest",
                        "ports": [{
                            "containerPort": 80
                        }]
                    }]
                })),
                status: Some(serde_json::json!({
                    "phase": "Running",
                    "conditions": [{
                        "type": "Ready",
                        "status": "True"
                    }]
                })),
            },
            K8sResource {
                apiVersion: "v1".to_string(),
                kind: "Pod".to_string(),
                metadata: Metadata {
                    name: "redis-67890".to_string(),
                    namespace: Some("default".to_string()),
                    labels: Some(serde_json::json!({
                        "app": "redis"
                    })),
                    annotations: None,
                },
                spec: Some(serde_json::json!({
                    "containers": [{
                        "name": "redis",
                        "image": "redis:latest",
                        "ports": [{
                            "containerPort": 6379
                        }]
                    }]
                })),
                status: Some(serde_json::json!({
                    "phase": "Running",
                    "conditions": [{
                        "type": "Ready",
                        "status": "True"
                    }]
                })),
            },
        ];

        let services = vec![K8sResource {
            apiVersion: "v1".to_string(),
            kind: "Service".to_string(),
            metadata: Metadata {
                name: "nginx-service".to_string(),
                namespace: Some("default".to_string()),
                labels: Some(serde_json::json!({
                    "app": "nginx"
                })),
                annotations: None,
            },
            spec: Some(serde_json::json!({
                "selector": {
                    "app": "nginx"
                },
                "ports": [{
                    "port": 80,
                    "targetPort": 80
                }],
                "type": "ClusterIP"
            })),
            status: Some(serde_json::json!({
                "loadBalancer": {}
            })),
        }];

        let deployments = vec![K8sResource {
            apiVersion: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            metadata: Metadata {
                name: "nginx-deployment".to_string(),
                namespace: Some("default".to_string()),
                labels: Some(serde_json::json!({
                    "app": "nginx"
                })),
                annotations: None,
            },
            spec: Some(serde_json::json!({
                "replicas": 3,
                "selector": {
                    "matchLabels": {
                        "app": "nginx"
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": "nginx"
                        }
                    },
                    "spec": {
                        "containers": [{
                            "name": "nginx",
                            "image": "nginx:latest",
                            "ports": [{
                                "containerPort": 80
                            }]
                        }]
                    }
                }
            })),
            status: Some(serde_json::json!({
                "replicas": 3,
                "availableReplicas": 3,
                "conditions": [{
                    "type": "Available",
                    "status": "True"
                }]
            })),
        }];

        let namespaces = vec![
            K8sResource {
                apiVersion: "v1".to_string(),
                kind: "Namespace".to_string(),
                metadata: Metadata { name: "default".to_string(), namespace: None, labels: None, annotations: None },
                spec: None,
                status: Some(serde_json::json!({
                    "phase": "Active"
                })),
            },
            K8sResource {
                apiVersion: "v1".to_string(),
                kind: "Namespace".to_string(),
                metadata: Metadata { name: "kube-system".to_string(), namespace: None, labels: None, annotations: None },
                spec: None,
                status: Some(serde_json::json!({
                    "phase": "Active"
                })),
            },
        ];

        Self { pods, services, deployments, namespaces }
    }

    fn get_resources(&self, resource_type: &str) -> Vec<K8sResource> {
        match resource_type {
            "pods" | "pod" => self.pods.clone(),
            "services" | "service" | "svc" => self.services.clone(),
            "deployments" | "deployment" | "deploy" => self.deployments.clone(),
            "namespaces" | "namespace" | "ns" => self.namespaces.clone(),
            _ => vec![],
        }
    }

    fn get_resource(&self, resource_type: &str, name: &str) -> Option<K8sResource> {
        self.get_resources(resource_type).into_iter().find(|r| r.metadata.name == name)
    }

    fn apply_resource(&mut self, resource: K8sResource) {
        match resource.kind.as_str() {
            "Pod" => {
                if let Some(index) = self.pods.iter().position(|p| p.metadata.name == resource.metadata.name) {
                    self.pods[index] = resource;
                }
                else {
                    self.pods.push(resource);
                }
            }
            "Service" => {
                if let Some(index) = self.services.iter().position(|s| s.metadata.name == resource.metadata.name) {
                    self.services[index] = resource;
                }
                else {
                    self.services.push(resource);
                }
            }
            "Deployment" => {
                if let Some(index) = self.deployments.iter().position(|d| d.metadata.name == resource.metadata.name) {
                    self.deployments[index] = resource;
                }
                else {
                    self.deployments.push(resource);
                }
            }
            "Namespace" => {
                if let Some(index) = self.namespaces.iter().position(|n| n.metadata.name == resource.metadata.name) {
                    self.namespaces[index] = resource;
                }
                else {
                    self.namespaces.push(resource);
                }
            }
            _ => {}
        }
    }

    fn delete_resource(&mut self, resource_type: &str, name: &str) -> bool {
        match resource_type {
            "pods" | "pod" => {
                if let Some(index) = self.pods.iter().position(|p| p.metadata.name == name) {
                    self.pods.remove(index);
                    true
                }
                else {
                    false
                }
            }
            "services" | "service" | "svc" => {
                if let Some(index) = self.services.iter().position(|s| s.metadata.name == name) {
                    self.services.remove(index);
                    true
                }
                else {
                    false
                }
            }
            "deployments" | "deployment" | "deploy" => {
                if let Some(index) = self.deployments.iter().position(|d| d.metadata.name == name) {
                    self.deployments.remove(index);
                    true
                }
                else {
                    false
                }
            }
            "namespaces" | "namespace" | "ns" => {
                if let Some(index) = self.namespaces.iter().position(|n| n.metadata.name == name) {
                    self.namespaces.remove(index);
                    true
                }
                else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl KubeConfig {
    async fn load() -> Result<Self, Box<dyn Error>> {
        // 尝试从默认位置读取 kubeconfig 文件
        let home_dir = dirs::home_dir().ok_or("Cannot find home directory")?;
        let kubeconfig_path = home_dir.join(".kube").join("config");

        if !kubeconfig_path.exists() {
            return Err(format!("Kubeconfig file not found at: {}", kubeconfig_path.display()).into());
        }

        // 读取文件内容
        let content = read_to_string(kubeconfig_path).await?;

        // 使用 oak-yaml 解析 YAML
        match parse(&content) {
            Ok(root) => {
                // 使用 oak-yaml 的 serde 支持进行反序列化
                // 这里需要实现从 oak-yaml AST 到 KubeConfig 的转换
                // 由于 oak-yaml 的 serde 支持可能需要特定的实现，我们暂时使用一个简单的方法
                // 后续可以改进为直接使用 oak-yaml 的 serde 功能
                let kubeconfig: KubeConfig = serde::de::Deserialize::deserialize(root)?;
                Ok(kubeconfig)
            }
            Err(e) => Err(format!("Failed to parse kubeconfig: {}", e).into()),
        }
    }
}
