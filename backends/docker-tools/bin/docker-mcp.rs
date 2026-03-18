use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_string_pretty};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

/// Docker MCP (Manifest Configuration Protocol) 命令行工具
///
/// 用于管理 Docker 镜像清单，支持创建、推送、拉取和管理多平台镜像
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建清单列表
    Create {
        /// 清单列表名称
        name: String,
        /// 镜像名称列表
        images: Vec<String>,
        /// 平台信息
        #[arg(long)]
        platform: Vec<String>,
    },
    /// 推送清单列表
    Push {
        /// 清单列表名称
        name: String,
        /// 跳过推送镜像
        #[arg(long)]
        skip_push: bool,
    },
    /// 拉取清单列表
    Pull {
        /// 清单列表名称
        name: String,
        /// 平台信息
        #[arg(long)]
        platform: Option<String>,
    },
    /// 检查清单列表
    Inspect {
        /// 清单列表名称
        name: String,
        /// 平台信息
        #[arg(long)]
        platform: Option<String>,
    },
    /// 删除清单列表
    Rm {
        /// 清单列表名称
        name: String,
    },
    /// 列出清单列表
    Ls {
        /// 仓库名称
        repository: Option<String>,
    },
    /// 修改清单列表
    Modify {
        /// 清单列表名称
        name: String,
        /// 添加镜像
        #[arg(long)]
        add: Vec<String>,
        /// 删除镜像
        #[arg(long)]
        remove: Vec<String>,
    },
}

/// 清单列表结构
#[derive(Debug, Serialize, Deserialize)]
struct ManifestList {
    schema_version: u32,
    media_type: String,
    manifests: Vec<ManifestEntry>,
}

/// 清单条目结构
#[derive(Debug, Serialize, Deserialize)]
struct ManifestEntry {
    media_type: String,
    size: u64,
    digest: String,
    platform: Platform,
}

/// 平台信息结构
#[derive(Debug, Serialize, Deserialize)]
struct Platform {
    architecture: String,
    os: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<Vec<String>>,
}

/// 镜像引用结构
struct ImageReference {
    registry: Option<String>,
    repository: String,
    tag: Option<String>,
}

impl FromStr for ImageReference {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        let mut registry = None;
        let mut repo_part = String::new();
        let mut tag = None;

        // 检查是否包含注册表
        if parts[0].contains(':') || parts[0].contains('.') {
            registry = Some(parts[0].to_string());
            repo_part = parts[1..].join("/");
        } else {
            repo_part = s.to_string();
        }

        // 检查是否包含标签
        let (repository, tag_part) = if let Some((repo, t)) = repo_part.split_once(':') {
            (repo.to_string(), Some(t.to_string()))
        } else {
            (repo_part, None)
        };

        if tag_part.is_some() {
            tag = tag_part;
        }

        Ok(ImageReference {
            registry,
            repository,
            tag,
        })
    }
}

/// 模拟 Docker 注册表客户端
struct RegistryClient {
    base_url: String,
    headers: HashMap<String, String>,
}

impl RegistryClient {
    fn new(registry: Option<&str>) -> Self {
        let base_url = match registry {
            Some(r) => format!("https://{}", r),
            None => "https://registry-1.docker.io".to_string(),
        };

        Self {
            base_url,
            headers: HashMap::new(),
        }
    }

    async fn get_manifest(&self, reference: &ImageReference) -> Result<Value, String> {
        // 模拟获取清单
        Ok(serde_json::json!({
            "schemaVersion": 2,
            "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
            "config": {
                "mediaType": "application/vnd.docker.container.image.v1+json",
                "size": 7023,
                "digest": "sha256:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
            },
            "layers": [
                {
                    "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
                    "size": 32654,
                    "digest": "sha256:yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy"
                }
            ]
        }))
    }

    async fn push_manifest(
        &self,
        reference: &ImageReference,
        manifest: &ManifestList,
    ) -> Result<(), String> {
        // 模拟推送清单
        Ok(())
    }

    async fn delete_manifest(&self, reference: &ImageReference) -> Result<(), String> {
        // 模拟删除清单
        Ok(())
    }

    async fn list_manifests(
        &self,
        repository: &str,
    ) -> Result<Vec<(String, String, Vec<String>)>, String> {
        // 模拟列出清单
        Ok(vec![
            (
                "myimage:latest".to_string(),
                "sha256:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                    .to_string(),
                vec!["linux/amd64".to_string(), "linux/arm64".to_string()],
            ),
            (
                "myimage:v1.0".to_string(),
                "sha256:yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy"
                    .to_string(),
                vec!["linux/amd64".to_string(), "linux/arm64".to_string()],
            ),
        ])
    }
}

/// 创建清单列表
async fn create_manifest_list(
    name: &str,
    images: &[String],
    platforms: &[String],
) -> Result<ManifestList, String> {
    let mut manifests = Vec::new();

    for (i, image) in images.iter().enumerate() {
        let image_ref: ImageReference = image.parse().map_err(|e| e)?;
        let client = RegistryClient::new(image_ref.registry.as_deref());

        // 获取镜像清单
        let manifest = client.get_manifest(&image_ref).await?;

        // 解析平台信息
        let platform = if i < platforms.len() {
            parse_platform(&platforms[i])?
        } else {
            // 默认平台信息
            Platform {
                architecture: "amd64".to_string(),
                os: "linux".to_string(),
                os_version: None,
                variant: None,
                features: None,
            }
        };

        // 创建清单条目
        let entry = ManifestEntry {
            media_type: "application/vnd.docker.distribution.manifest.v2+json".to_string(),
            size: 7023,                        // 模拟大小
            digest: format!("sha256:{:x}", i), // 模拟摘要
            platform,
        };

        manifests.push(entry);
    }

    // 创建清单列表
    let manifest_list = ManifestList {
        schema_version: 2,
        media_type: "application/vnd.docker.distribution.manifest.list.v2+json".to_string(),
        manifests,
    };

    Ok(manifest_list)
}

/// 解析平台信息
fn parse_platform(platform_str: &str) -> Result<Platform, String> {
    let parts: Vec<&str> = platform_str.split('/').collect();
    if parts.len() < 2 {
        return Err("Invalid platform format".to_string());
    }

    let os = parts[0].to_string();
    let architecture = parts[1].to_string();
    let mut os_version = None;
    let mut variant = None;

    // 解析额外信息
    for part in &parts[2..] {
        if part.starts_with("osversion=") {
            os_version = Some(part.trim_start_matches("osversion=").to_string());
        } else if part.starts_with("variant=") {
            variant = Some(part.trim_start_matches("variant=").to_string());
        }
    }

    Ok(Platform {
        architecture,
        os,
        os_version,
        variant,
        features: None,
    })
}

/// 推送清单列表
async fn push_manifest_list(name: &str, skip_push: bool) -> Result<(), String> {
    let image_ref: ImageReference = name.parse().map_err(|e| e)?;
    let client = RegistryClient::new(image_ref.registry.as_deref());

    // 读取清单列表文件
    let manifest_list_path = format!("{}.json", name.replace('/', "_"));
    let mut file = File::open(&manifest_list_path).map_err(|e| e.to_string())?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| e.to_string())?;

    let manifest_list: ManifestList = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    // 推送清单列表
    client.push_manifest(&image_ref, &manifest_list).await?;

    if !skip_push {
        // 推送关联的镜像（如果需要）
        println!("Pushing associated images...");
    }

    Ok(())
}

/// 拉取清单列表
async fn pull_manifest_list(name: &str, platform: Option<&str>) -> Result<ManifestList, String> {
    let image_ref: ImageReference = name.parse().map_err(|e| e)?;
    let client = RegistryClient::new(image_ref.registry.as_deref());

    // 模拟拉取清单列表
    let manifest_list = ManifestList {
        schema_version: 2,
        media_type: "application/vnd.docker.distribution.manifest.list.v2+json".to_string(),
        manifests: vec![
            ManifestEntry {
                media_type: "application/vnd.docker.distribution.manifest.v2+json".to_string(),
                size: 7023,
                digest: "sha256:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                    .to_string(),
                platform: Platform {
                    architecture: "amd64".to_string(),
                    os: "linux".to_string(),
                    os_version: None,
                    variant: None,
                    features: None,
                },
            },
            ManifestEntry {
                media_type: "application/vnd.docker.distribution.manifest.v2+json".to_string(),
                size: 7023,
                digest: "sha256:yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy"
                    .to_string(),
                platform: Platform {
                    architecture: "arm64".to_string(),
                    os: "linux".to_string(),
                    os_version: None,
                    variant: None,
                    features: None,
                },
            },
        ],
    };

    // 如果指定了平台，过滤出对应平台的清单
    if let Some(platform_str) = platform {
        let target_platform = parse_platform(platform_str)?;
        let filtered_manifests = manifest_list
            .manifests
            .into_iter()
            .filter(|m| {
                m.platform.architecture == target_platform.architecture
                    && m.platform.os == target_platform.os
            })
            .collect();

        Ok(ManifestList {
            schema_version: manifest_list.schema_version,
            media_type: manifest_list.media_type,
            manifests: filtered_manifests,
        })
    } else {
        Ok(manifest_list)
    }
}

/// 检查清单列表
async fn inspect_manifest_list(name: &str, platform: Option<&str>) -> Result<ManifestList, String> {
    pull_manifest_list(name, platform).await
}

/// 删除清单列表
async fn remove_manifest_list(name: &str) -> Result<(), String> {
    let image_ref: ImageReference = name.parse().map_err(|e| e)?;
    let client = RegistryClient::new(image_ref.registry.as_deref());

    client.delete_manifest(&image_ref).await
}

/// 列出清单列表
async fn list_manifest_lists(
    repository: Option<&str>,
) -> Result<Vec<(String, String, Vec<String>)>, String> {
    let client = RegistryClient::new(None);

    match repository {
        Some(repo) => client.list_manifests(repo).await,
        None => Ok(vec![]),
    }
}

/// 修改清单列表
async fn modify_manifest_list(
    name: &str,
    add: &[String],
    remove: &[String],
) -> Result<ManifestList, String> {
    // 读取现有清单列表
    let manifest_list_path = format!("{}.json", name.replace('/', "_"));
    let mut file = File::open(&manifest_list_path).map_err(|e| e.to_string())?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| e.to_string())?;

    let mut manifest_list: ManifestList =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    // 添加新镜像
    for image in add {
        let image_ref: ImageReference = image.parse().map_err(|e| e)?;
        let client = RegistryClient::new(image_ref.registry.as_deref());

        // 获取镜像清单
        let manifest = client.get_manifest(&image_ref).await?;

        // 创建清单条目
        let entry = ManifestEntry {
            media_type: "application/vnd.docker.distribution.manifest.v2+json".to_string(),
            size: 7023,                                                    // 模拟大小
            digest: format!("sha256:{:x}", manifest_list.manifests.len()), // 模拟摘要
            platform: Platform {
                architecture: "amd64".to_string(),
                os: "linux".to_string(),
                os_version: None,
                variant: None,
                features: None,
            },
        };

        manifest_list.manifests.push(entry);
    }

    // 删除指定镜像
    // 这里简化处理，实际应该根据镜像名称或平台信息删除
    if !remove.is_empty() {
        // 模拟删除
        if !manifest_list.manifests.is_empty() {
            manifest_list.manifests.pop();
        }
    }

    // 保存修改后的清单列表
    let updated_content = to_string_pretty(&manifest_list).map_err(|e| e.to_string())?;
    let mut file = File::create(&manifest_list_path).map_err(|e| e.to_string())?;
    file.write_all(updated_content.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(manifest_list)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            name,
            images,
            platform,
        } => {
            println!("Creating manifest list: {}", name);
            match create_manifest_list(&name, &images, &platform).await {
                Ok(manifest_list) => {
                    // 保存清单列表到文件
                    let manifest_list_path = format!("{}.json", name.replace('/', "_"));
                    let content = to_string_pretty(&manifest_list).unwrap();
                    let mut file = File::create(&manifest_list_path).unwrap();
                    file.write_all(content.as_bytes()).unwrap();

                    println!("Manifest list created successfully");
                    println!("Name: {}", name);
                    println!("Saved to: {}", manifest_list_path);
                }
                Err(e) => {
                    println!("Error creating manifest list: {}", e);
                }
            }
        }
        Commands::Push { name, skip_push } => {
            println!("Pushing manifest list: {}", name);
            match push_manifest_list(&name, skip_push).await {
                Ok(_) => {
                    println!("Manifest list pushed successfully");
                }
                Err(e) => {
                    println!("Error pushing manifest list: {}", e);
                }
            }
        }
        Commands::Pull { name, platform } => {
            println!("Pulling manifest list: {}", name);
            match pull_manifest_list(&name, platform.as_deref()).await {
                Ok(manifest_list) => {
                    // 保存拉取的清单列表
                    let manifest_list_path = format!("{}.json", name.replace('/', "_"));
                    let content = to_string_pretty(&manifest_list).unwrap();
                    let mut file = File::create(&manifest_list_path).unwrap();
                    file.write_all(content.as_bytes()).unwrap();

                    println!("Manifest list pulled successfully");
                    println!("Saved to: {}", manifest_list_path);
                }
                Err(e) => {
                    println!("Error pulling manifest list: {}", e);
                }
            }
        }
        Commands::Inspect { name, platform } => {
            println!("Inspecting manifest list: {}", name);
            match inspect_manifest_list(&name, platform.as_deref()).await {
                Ok(manifest_list) => {
                    println!("Manifest list details:");
                    println!("{}", to_string_pretty(&manifest_list).unwrap());
                }
                Err(e) => {
                    println!("Error inspecting manifest list: {}", e);
                }
            }
        }
        Commands::Rm { name } => {
            println!("Removing manifest list: {}", name);
            match remove_manifest_list(&name).await {
                Ok(_) => {
                    // 删除本地文件
                    let manifest_list_path = format!("{}.json", name.replace('/', "_"));
                    if Path::new(&manifest_list_path).exists() {
                        std::fs::remove_file(&manifest_list_path).unwrap();
                    }
                    println!("Manifest list removed successfully");
                }
                Err(e) => {
                    println!("Error removing manifest list: {}", e);
                }
            }
        }
        Commands::Ls { repository } => {
            println!("Listing manifest lists");
            if let Some(repo) = &repository {
                println!("Repository: {}", repo);
            }
            match list_manifest_lists(repository.as_deref()).await {
                Ok(manifests) => {
                    println!("Manifest lists:");
                    println!(
                        "NAME                                    DIGEST                                                                  PLATFORMS"
                    );
                    for (name, digest, platforms) in manifests {
                        println!("{:<40} {:<70} {:?}", name, digest, platforms.join(", "));
                    }
                }
                Err(e) => {
                    println!("Error listing manifest lists: {}", e);
                }
            }
        }
        Commands::Modify { name, add, remove } => {
            println!("Modifying manifest list: {}", name);
            match modify_manifest_list(&name, &add, &remove).await {
                Ok(manifest_list) => {
                    println!("Manifest list modified successfully");
                    println!("Updated manifest list:");
                    println!("{}", to_string_pretty(&manifest_list).unwrap());
                }
                Err(e) => {
                    println!("Error modifying manifest list: {}", e);
                }
            }
        }
    }
}
