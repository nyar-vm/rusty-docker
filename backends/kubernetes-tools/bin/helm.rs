use chrono;
use clap::{Parser, Subcommand};
use dirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tokio;

/// Helm 仓库配置
#[derive(Debug, Deserialize, Serialize)]
struct HelmRepository {
    name: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cert_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ca_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insecure_skip_tls_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bearer_token: Option<String>,
}

/// Helm 仓库配置文件
#[derive(Debug, Deserialize, Serialize)]
struct HelmRepositories {
    repositories: Vec<HelmRepository>,
}

/// Helm chart 信息
#[derive(Debug, Deserialize, Serialize)]
struct HelmChart {
    name: String,
    version: String,
    description: String,
    app_version: String,
    urls: Vec<String>,
}

/// Helm 命令行工具
///
/// 用于管理 Kubernetes 应用程序的包管理工具
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 添加 Helm 仓库
    Repo {
        #[command(subcommand)]
        repo_command: RepoCommands,
    },
    /// 安装 Helm chart
    Install {
        /// 发布名称
        name: String,
        /// Chart 路径或名称
        chart: String,
        /// 设置值
        #[arg(long, short = 'f')]
        values: Option<String>,
        /// 设置单个值
        #[arg(long)]
        set: Vec<String>,
        /// 命名空间
        #[arg(long, default_value = "default")]
        namespace: String,
    },
    /// 升级已安装的 Helm 发布
    Upgrade {
        /// 发布名称
        name: String,
        /// Chart 路径或名称
        chart: String,
        /// 设置值
        #[arg(long, short = 'f')]
        values: Option<String>,
        /// 设置单个值
        #[arg(long)]
        set: Vec<String>,
        /// 命名空间
        #[arg(long, default_value = "default")]
        namespace: String,
    },
    /// 卸载 Helm 发布
    Uninstall {
        /// 发布名称
        name: String,
        /// 命名空间
        #[arg(long, default_value = "default")]
        namespace: String,
    },
    /// 列出已安装的 Helm 发布
    List {
        /// 命名空间
        #[arg(long, default_value = "default")]
        namespace: String,
        /// 显示所有命名空间的发布
        #[arg(long)]
        all_namespaces: bool,
    },
    /// 查看 Helm 发布状态
    Status {
        /// 发布名称
        name: String,
        /// 命名空间
        #[arg(long, default_value = "default")]
        namespace: String,
    },
    /// 拉取 Helm chart
    Pull {
        /// Chart 名称
        chart: String,
        /// 输出目录
        #[arg(long, short = 'd')]
        destination: Option<String>,
        /// 版本
        #[arg(long)]
        version: Option<String>,
    },
    /// 搜索 Helm chart
    Search {
        #[command(subcommand)]
        search_command: SearchCommands,
    },
    /// 模板渲染
    Template {
        /// 发布名称
        name: String,
        /// Chart 路径或名称
        chart: String,
        /// 设置值
        #[arg(long, short = 'f')]
        values: Option<String>,
        /// 设置单个值
        #[arg(long)]
        set: Vec<String>,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    /// 添加仓库
    Add {
        /// 仓库名称
        name: String,
        /// 仓库 URL
        url: String,
    },
    /// 更新仓库
    Update,
    /// 列出仓库
    List,
    /// 删除仓库
    Remove {
        /// 仓库名称
        name: String,
    },
}

#[derive(Subcommand)]
enum SearchCommands {
    /// 搜索仓库
    Repo {
        /// 搜索关键词
        query: String,
    },
    /// 搜索本地 chart
    Hub {
        /// 搜索关键词
        query: String,
    },
}

/// 获取 Helm 配置目录
fn get_helm_config_dir() -> String {
    let home_dir = dirs::home_dir().expect("无法获取用户主目录");
    let helm_dir = home_dir.join(".helm");
    helm_dir.to_str().expect("无法将路径转换为字符串").to_string()
}

/// 获取 Helm 仓库配置文件路径
fn get_repositories_file() -> String {
    let config_dir = get_helm_config_dir();
    let repos_file = Path::new(&config_dir).join("repositories.yaml");
    repos_file.to_str().expect("无法将路径转换为字符串").to_string()
}

/// 初始化 Helm 配置目录
fn init_helm_config() -> Result<(), String> {
    let config_dir = get_helm_config_dir();
    let repo_dir = Path::new(&config_dir).join("repository");
    let cache_dir = repo_dir.join("cache");
    let local_dir = repo_dir.join("local");

    fs::create_dir_all(&cache_dir).map_err(|e| format!("无法创建 Helm 缓存目录: {}", e))?;
    fs::create_dir_all(&local_dir).map_err(|e| format!("无法创建 Helm 本地仓库目录: {}", e))?;

    // 如果仓库配置文件不存在，创建一个空的
    let repos_file = get_repositories_file();
    if !Path::new(&repos_file).exists() {
        let yaml = r#"repositories: []
"#;
        fs::write(&repos_file, yaml).map_err(|e| format!("无法写入仓库配置文件: {}", e))?;
    }

    Ok(())
}

/// 读取 Helm 仓库配置
fn read_repositories() -> Result<HelmRepositories, String> {
    let repos_file = get_repositories_file();
    if !Path::new(&repos_file).exists() {
        return Err("仓库配置文件不存在".to_string());
    }
    // Mock 实现，返回默认配置
    Ok(HelmRepositories { repositories: vec![] })
}

/// 写入 Helm 仓库配置
fn write_repositories(repos: &HelmRepositories) -> Result<(), String> {
    let repos_file = get_repositories_file();
    // Mock 实现，创建一个简单的仓库配置文件
    let yaml = r#"repositories: []
"#;
    fs::write(&repos_file, yaml).map_err(|e| format!("无法写入仓库配置文件: {}", e))
}

/// 从仓库获取 chart 信息
async fn get_charts_from_repo(repo_url: &str) -> Result<Vec<HelmChart>, String> {
    // 模拟实现，直接返回模拟数据
    Ok(vec![
        HelmChart {
            name: "nginx".to_string(),
            version: "1.2.3".to_string(),
            description: "NGINX is a free, open-source, high-performance HTTP server and reverse proxy.".to_string(),
            app_version: "1.21.6".to_string(),
            urls: vec![format!("{}/nginx-1.2.3.tgz", repo_url)],
        },
        HelmChart {
            name: "mysql".to_string(),
            version: "8.0.31".to_string(),
            description: "MySQL is a widely used, open-source relational database management system.".to_string(),
            app_version: "8.0.31".to_string(),
            urls: vec![format!("{}/mysql-8.0.31.tgz", repo_url)],
        },
    ])
}

#[tokio::main]
async fn main() {
    // 初始化 Helm 配置
    if let Err(err) = init_helm_config() {
        println!("Error: {}", err);
        return;
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Repo { repo_command } => match repo_command {
            RepoCommands::Add { name, url } => {
                println!("Adding Helm repository");
                println!("Name: {}", name);
                println!("URL: {}", url);

                // 读取现有仓库配置
                match read_repositories() {
                    Ok(mut repos) => {
                        // 检查仓库是否已存在
                        if repos.repositories.iter().any(|r| r.name == name) {
                            println!("Error: repository with name '{}' already exists", name);
                            return;
                        }

                        // 添加新仓库
                        repos.repositories.push(HelmRepository {
                            name,
                            url,
                            cert_file: None,
                            key_file: None,
                            ca_file: None,
                            insecure_skip_tls_verify: None,
                            username: None,
                            password: None,
                            bearer_token: None,
                        });

                        // 写入配置
                        match write_repositories(&repos) {
                            Ok(_) => {
                                println!("Repository added successfully");
                            }
                            Err(err) => {
                                println!("Error: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            RepoCommands::Update => {
                println!("Updating Helm repositories");
                println!("Hang tight while we grab the latest from your chart repositories...");

                // 读取仓库配置
                match read_repositories() {
                    Ok(repos) => {
                        // 模拟更新每个仓库
                        for repo in &repos.repositories {
                            println!("...Successfully got an update from the '{}' chart repository", repo.name);
                        }

                        println!("Update Complete.");
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            RepoCommands::List => {
                println!("Listing Helm repositories");
                println!("NAME            URL");

                // 读取仓库配置
                match read_repositories() {
                    Ok(repos) => {
                        for repo in repos.repositories {
                            println!("{:<15} {}", repo.name, repo.url);
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            RepoCommands::Remove { name } => {
                println!("Removing Helm repository");
                println!("Name: {}", name);

                // 读取现有仓库配置
                match read_repositories() {
                    Ok(mut repos) => {
                        // 检查仓库是否存在
                        let original_len = repos.repositories.len();
                        repos.repositories.retain(|r| r.name != name);

                        if repos.repositories.len() == original_len {
                            println!("Error: repository with name '{}' not found", name);
                            return;
                        }

                        // 写入配置
                        match write_repositories(&repos) {
                            Ok(_) => {
                                println!("Repository removed successfully");
                            }
                            Err(err) => {
                                println!("Error: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        },
        Commands::Install { name, chart, values, set, namespace } => {
            println!("Installing Helm chart");
            println!("Release name: {}", name);
            println!("Chart: {}", chart);
            if let Some(v) = &values {
                println!("Values file: {}", v);
            }
            for s in &set {
                println!("Set: {}", s);
            }
            println!("Namespace: {}", namespace);
            let now = chrono::Utc::now().to_rfc3339();
            println!("Release '{}' has been installed successfully!", name);
            println!("NAME: {}", name);
            println!("LAST DEPLOYED: {}", now);
            println!("NAMESPACE: {}", namespace);
            println!("STATUS: deployed");
            println!("REVISION: 1");
            println!("TEST SUITE: None");
        }
        Commands::Upgrade { name, chart, values, set, namespace } => {
            println!("Upgrading Helm release");
            println!("Release name: {}", name);
            println!("Chart: {}", chart);
            if let Some(v) = &values {
                println!("Values file: {}", v);
            }
            for s in &set {
                println!("Set: {}", s);
            }
            println!("Namespace: {}", namespace);
            let now = chrono::Utc::now().to_rfc3339();
            println!("Release '{}' has been upgraded successfully!", name);
            println!("NAME: {}", name);
            println!("LAST DEPLOYED: {}", now);
            println!("NAMESPACE: {}", namespace);
            println!("STATUS: deployed");
            println!("REVISION: 2");
            println!("TEST SUITE: None");
        }
        Commands::Uninstall { name, namespace } => {
            println!("Uninstalling Helm release");
            println!("Release name: {}", name);
            println!("Namespace: {}", namespace);
            println!("Release '{}' uninstalled successfully", name);
        }
        Commands::List { namespace, all_namespaces } => {
            println!("Listing Helm releases");
            if all_namespaces {
                println!("All namespaces");
            }
            else {
                println!("Namespace: {}", namespace);
            }
            println!(
                "NAME            NAMESPACE       REVISION        UPDATED                                 STATUS          CHART                   APP VERSION"
            );
            println!(
                "nginx           default         1               2024-01-01 12:00:00.000000000 +0000 UTC deployed        nginx-1.2.3             1.21.6"
            );
            println!(
                "mysql           default         2               2024-01-02 12:00:00.000000000 +0000 UTC deployed        mysql-8.0.31            8.0.31"
            );
        }
        Commands::Status { name, namespace } => {
            println!("Checking Helm release status");
            println!("Release name: {}", name);
            println!("Namespace: {}", namespace);
            println!("NAME: {}", name);
            println!("LAST DEPLOYED: 2024-01-01 12:00:00.000000000 +0000 UTC");
            println!("NAMESPACE: {}", namespace);
            println!("STATUS: deployed");
            println!("REVISION: 1");
            println!("TEST SUITE: None");
            println!("NOTES:");
            println!("1. Get the application URL by running:");
            println!("   kubectl get svc --namespace {} {}-nginx", namespace, name);
        }
        Commands::Pull { chart, destination, version } => {
            println!("Pulling Helm chart");
            println!("Chart: {}", chart);
            if let Some(d) = &destination {
                println!("Destination: {}", d);
            }
            if let Some(v) = &version {
                println!("Version: {}", v);
            }
            println!("Pulled chart successfully");
        }
        Commands::Search { search_command } => match search_command {
            SearchCommands::Repo { query } => {
                println!("Searching Helm repositories for '{}'", query);
                println!("NAME                            CHART VERSION   APP VERSION     DESCRIPTION");
                println!(
                    "stable/nginx                    1.2.3           1.21.6          NGINX is a free, open-source, high-performance HTTP server and reverse proxy."
                );
                println!(
                    "bitnami/nginx                   13.2.2          1.21.6          NGINX is a free, open-source, high-performance HTTP server and reverse proxy."
                );
            }
            SearchCommands::Hub { query } => {
                println!("Searching Helm Hub for '{}'", query);
                println!("NAME                            CHART VERSION   APP VERSION     DESCRIPTION");
                println!(
                    "helm/nginx                      1.2.3           1.21.6          NGINX is a free, open-source, high-performance HTTP server and reverse proxy."
                );
                println!(
                    "kubernetes/nginx                1.0.0           1.21.6          NGINX is a free, open-source, high-performance HTTP server and reverse proxy."
                );
            }
        },
        Commands::Template { name, chart, values, set } => {
            println!("Rendering Helm chart template");
            println!("Release name: {}", name);
            println!("Chart: {}", chart);
            if let Some(v) = &values {
                println!("Values file: {}", v);
            }
            for s in &set {
                println!("Set: {}", s);
            }
            println!("---");
            println!("# Source: nginx/templates/service.yaml");
            println!("apiVersion: v1");
            println!("kind: Service");
            println!("metadata:");
            println!("  name: {}-nginx", name);
            println!("  labels:");
            println!("    app.kubernetes.io/name: nginx");
            println!("    app.kubernetes.io/instance: {}", name);
            println!("spec:");
            println!("  type: ClusterIP");
            println!("  ports:");
            println!("    - port: 80");
            println!("      targetPort: http");
            println!("      protocol: TCP");
            println!("      name: http");
            println!("  selector:");
            println!("    app.kubernetes.io/name: nginx");
            println!("    app.kubernetes.io/instance: {}", name);
        }
    }
}
