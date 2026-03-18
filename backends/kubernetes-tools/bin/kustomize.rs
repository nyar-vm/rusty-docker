use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

/// Kustomization 配置
#[derive(Debug, Deserialize, Serialize)]
struct Kustomization {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patches: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replicas: Option<Vec<Replica>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<Label>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<Vec<Annotation>>,
}

/// Replica 配置
#[derive(Debug, Deserialize, Serialize)]
struct Replica {
    name: String,
    count: i32,
}

/// Label 配置
#[derive(Debug, Deserialize, Serialize)]
struct Label {
    pairs: String,
    includeSelectors: Option<bool>,
    includeTemplates: Option<bool>,
}

/// Annotation 配置
#[derive(Debug, Deserialize, Serialize)]
struct Annotation {
    pairs: String,
}

/// Kustomize 命令行工具
///
/// 用于通过补丁定制 Kubernetes 资源，支持基础配置和覆盖
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 构建定制化的 Kubernetes 资源
    Build {
        /// kustomization 目录路径
        path: Option<String>,
        /// 输出格式 (yaml|json)
        #[arg(long, default_value = "yaml")]
        output: String,
        /// 包含资源
        #[arg(long)]
        include: Vec<String>,
        /// 排除资源
        #[arg(long)]
        exclude: Vec<String>,
    },
    /// 创建新的 kustomization 目录
    Create {
        /// 目录路径
        path: String,
        /// 基础目录
        #[arg(long)]
        base: Option<String>,
        /// 资源文件
        #[arg(long)]
        resources: Vec<String>,
    },
    /// 编辑 kustomization 文件
    Edit {
        /// 操作类型
        #[command(subcommand)]
        edit_command: EditCommands,
    },
    /// 验证 kustomization 配置
    Validate {
        /// kustomization 目录路径
        path: Option<String>,
    },
    /// 列出资源
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum EditCommands {
    /// 添加资源
    Add {
        /// 资源文件路径
        resources: Vec<String>,
    },
    /// 添加基础
    Set {
        /// 设置类型
        #[command(subcommand)]
        set_command: SetCommands,
    },
    /// 添加补丁
    AddPatch {
        /// 补丁文件路径
        patches: Vec<String>,
    },
    /// 添加标签
    AddLabel {
        /// 标签键值对
        labels: Vec<String>,
    },
    /// 添加注解
    AddAnnotation {
        /// 注解键值对
        annotations: Vec<String>,
    },
}

#[derive(Subcommand)]
enum SetCommands {
    /// 设置命名空间
    Namespace {
        /// 命名空间名称
        namespace: String,
    },
    /// 设置镜像
    Image {
        /// 镜像规格
        images: Vec<String>,
    },
    /// 设置 replicas
    Replicas {
        /// 资源名称和副本数
        replicas: String,
    },
    /// 设置基础
    Base {
        /// 基础目录路径
        base: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// 列出所有资源
    View {
        /// kustomization 目录路径
        path: Option<String>,
    },
    /// 查看 diff
    Diff {
        /// 基础目录路径
        base: String,
        /// 覆盖目录路径
        overlay: String,
    },
}

/// 获取 kustomization.yaml 文件路径
fn get_kustomization_path(path: Option<&str>) -> String {
    let base_path = path.map_or(".", |p| p);
    let kustomization_path = Path::new(base_path).join("kustomization.yaml");
    kustomization_path
        .to_str()
        .expect("无法将路径转换为字符串")
        .to_string()
}

/// 读取 kustomization.yaml 文件
fn read_kustomization(path: Option<&str>) -> Result<Kustomization, String> {
    let kustomization_path = get_kustomization_path(path);

    if !Path::new(&kustomization_path).exists() {
        return Err(format!(
            "kustomization.yaml 文件不存在: {}",
            kustomization_path
        ));
    }

    let content = fs::read_to_string(&kustomization_path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&content).map_err(|e| e.to_string())
}

/// 写入 kustomization.yaml 文件
fn write_kustomization(kustomization: &Kustomization, path: Option<&str>) -> Result<(), String> {
    let kustomization_path = get_kustomization_path(path);
    let yaml = serde_yaml::to_string(kustomization).map_err(|e| e.to_string())?;
    fs::write(&kustomization_path, yaml).map_err(|e| e.to_string())
}

/// 构建定制化的 Kubernetes 资源
fn build_resources(
    path: Option<&str>,
    _output: &String,
    _include: &[String],
    _exclude: &[String],
) -> Result<String, String> {
    // 读取 kustomization.yaml
    let _kustomization = read_kustomization(path)?;

    // 模拟构建过程
    let mut resources = String::new();

    // 添加 ConfigMap
    resources.push_str("apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: example-config\n  namespace: default\ndata:\n  key: value\n---\n");

    // 添加 Deployment
    resources.push_str("apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: example-deployment\n  namespace: default\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: example\n  template:\n    metadata:\n      labels:\n        app: example\n    spec:\n      containers:\n      - name: example\n        image: nginx:latest\n        ports:\n        - containerPort: 80\n");

    Ok(resources)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            path,
            output,
            include,
            exclude,
        } => {
            println!("Building kustomization");
            let base_path = path.as_deref().unwrap_or(".");
            println!("Path: {}", base_path);
            println!("Output format: {}", output);
            for inc in &include {
                println!("Include: {}", inc);
            }
            for exc in &exclude {
                println!("Exclude: {}", exc);
            }

            match build_resources(path.as_deref(), &output, &include, &exclude) {
                Ok(resources) => {
                    println!("Building resources...");
                    println!("{}", resources);
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
        Commands::Create {
            path,
            base,
            resources,
        } => {
            println!("Creating kustomization directory");
            println!("Path: {}", path);

            // 创建目录
            if let Err(err) = fs::create_dir_all(&path) {
                println!("Error: 无法创建目录: {}", err);
                return;
            }

            // 创建 kustomization.yaml
            let mut kustomization = Kustomization {
                api_version: "kustomize.config.k8s.io/v1beta1".to_string(),
                kind: "Kustomization".to_string(),
                resources: Some(resources),
                bases: base.map(|b| vec![b]),
                patches: None,
                namespace: None,
                images: None,
                replicas: None,
                labels: None,
                annotations: None,
            };

            // 如果没有指定资源，添加默认资源
            if kustomization
                .resources
                .as_ref()
                .unwrap_or(&vec![])
                .is_empty()
            {
                kustomization.resources = Some(vec![
                    "service.yaml".to_string(),
                    "deployment.yaml".to_string(),
                ]);
            }

            match write_kustomization(&kustomization, Some(path.as_str())) {
                Ok(_) => {
                    println!("Created kustomization.yaml at {}/kustomization.yaml", path);
                }
                Err(err) => {
                    println!("Error: 无法创建 kustomization.yaml: {}", err);
                }
            }
        }
        Commands::Edit { edit_command } => match edit_command {
            EditCommands::Add { resources } => {
                println!("Adding resources");
                for resource in &resources {
                    println!("Resource: {}", resource);
                }

                match read_kustomization(None) {
                    Ok(mut kustomization) => {
                        let mut existing_resources = kustomization.resources.unwrap_or(vec![]);
                        existing_resources.extend(resources);
                        kustomization.resources = Some(existing_resources);

                        match write_kustomization(&kustomization, None) {
                            Ok(_) => {
                                println!("Resources added successfully");
                            }
                            Err(err) => {
                                println!("Error: 无法更新 kustomization.yaml: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            EditCommands::Set { set_command } => match set_command {
                SetCommands::Namespace { namespace } => {
                    println!("Setting namespace: {}", namespace);

                    match read_kustomization(None) {
                        Ok(mut kustomization) => {
                            kustomization.namespace = Some(namespace);

                            match write_kustomization(&kustomization, None) {
                                Ok(_) => {
                                    println!("Namespace set successfully");
                                }
                                Err(err) => {
                                    println!("Error: 无法更新 kustomization.yaml: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            println!("Error: {}", err);
                        }
                    }
                }
                SetCommands::Image { images } => {
                    println!("Setting images");
                    for image in &images {
                        println!("Image: {}", image);
                    }

                    match read_kustomization(None) {
                        Ok(mut kustomization) => {
                            kustomization.images = Some(images);

                            match write_kustomization(&kustomization, None) {
                                Ok(_) => {
                                    println!("Images set successfully");
                                }
                                Err(err) => {
                                    println!("Error: 无法更新 kustomization.yaml: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            println!("Error: {}", err);
                        }
                    }
                }
                SetCommands::Replicas { replicas } => {
                    println!("Setting replicas: {}", replicas);

                    // 解析 replicas 字符串，格式为 name=count
                    let parts: Vec<&str> = replicas.split('=').collect();
                    if parts.len() != 2 {
                        println!("Error: 无效的 replicas 格式，应该是 name=count");
                        return;
                    }

                    let name = parts[0].to_string();
                    let count = parts[1].parse::<i32>().map_err(|e| e.to_string());

                    if let Err(err) = count {
                        println!("Error: 无效的副本数: {}", err);
                        return;
                    }

                    let count = count.unwrap();

                    match read_kustomization(None) {
                        Ok(mut kustomization) => {
                            let mut existing_replicas = kustomization.replicas.unwrap_or(vec![]);
                            existing_replicas.push(Replica { name, count });
                            kustomization.replicas = Some(existing_replicas);

                            match write_kustomization(&kustomization, None) {
                                Ok(_) => {
                                    println!("Replicas set successfully");
                                }
                                Err(err) => {
                                    println!("Error: 无法更新 kustomization.yaml: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            println!("Error: {}", err);
                        }
                    }
                }
                SetCommands::Base { base } => {
                    println!("Setting base: {}", base);

                    match read_kustomization(None) {
                        Ok(mut kustomization) => {
                            kustomization.bases = Some(vec![base]);

                            match write_kustomization(&kustomization, None) {
                                Ok(_) => {
                                    println!("Base set successfully");
                                }
                                Err(err) => {
                                    println!("Error: 无法更新 kustomization.yaml: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            println!("Error: {}", err);
                        }
                    }
                }
            },
            EditCommands::AddPatch { patches } => {
                println!("Adding patches");
                for patch in &patches {
                    println!("Patch: {}", patch);
                }

                match read_kustomization(None) {
                    Ok(mut kustomization) => {
                        let mut existing_patches = kustomization.patches.unwrap_or(vec![]);
                        existing_patches.extend(patches);
                        kustomization.patches = Some(existing_patches);

                        match write_kustomization(&kustomization, None) {
                            Ok(_) => {
                                println!("Patches added successfully");
                            }
                            Err(err) => {
                                println!("Error: 无法更新 kustomization.yaml: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            EditCommands::AddLabel { labels } => {
                println!("Adding labels");
                for label in &labels {
                    println!("Label: {}", label);
                }

                match read_kustomization(None) {
                    Ok(mut kustomization) => {
                        let mut existing_labels = kustomization.labels.unwrap_or(vec![]);
                        for label in labels {
                            existing_labels.push(Label {
                                pairs: label.clone(),
                                includeSelectors: None,
                                includeTemplates: None,
                            });
                        }
                        kustomization.labels = Some(existing_labels);

                        match write_kustomization(&kustomization, None) {
                            Ok(_) => {
                                println!("Labels added successfully");
                            }
                            Err(err) => {
                                println!("Error: 无法更新 kustomization.yaml: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            EditCommands::AddAnnotation { annotations } => {
                println!("Adding annotations");
                for annotation in &annotations {
                    println!("Annotation: {}", annotation);
                }

                match read_kustomization(None) {
                    Ok(mut kustomization) => {
                        let mut existing_annotations = kustomization.annotations.unwrap_or(vec![]);
                        for annotation in annotations {
                            existing_annotations.push(Annotation {
                                pairs: annotation.clone(),
                            });
                        }
                        kustomization.annotations = Some(existing_annotations);

                        match write_kustomization(&kustomization, None) {
                            Ok(_) => {
                                println!("Annotations added successfully");
                            }
                            Err(err) => {
                                println!("Error: 无法更新 kustomization.yaml: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        },
        Commands::Validate { path } => {
            println!("Validating kustomization");
            let base_path = path.as_deref().unwrap_or(".");
            println!("Path: {}", base_path);

            match read_kustomization(path.as_deref()) {
                Ok(_) => {
                    println!("Validation successful");
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
        Commands::Config { config_command } => match config_command {
            ConfigCommands::View { path } => {
                println!("Viewing config");
                let base_path = path.as_deref().unwrap_or(".");
                println!("Path: {}", base_path);

                match read_kustomization(path.as_deref()) {
                    Ok(kustomization) => {
                        println!("Config content:");
                        let yaml = serde_yaml::to_string(&kustomization)
                            .expect("无法序列化 kustomization");
                        println!("{}", yaml);
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            ConfigCommands::Diff { base, overlay } => {
                println!("Viewing diff");
                println!("Base: {}", base);
                println!("Overlay: {}", overlay);
                println!("Diff result:");
                println!("--- base/deployment.yaml");
                println!("+++ overlay/deployment.yaml");
                println!("@@ -10,7 +10,7 @@");
                println!("   replicas: 1");
                println!("+++ replicas: 3");
            }
        },
    }
}
