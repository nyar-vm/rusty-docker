use base64;
use clap::{Parser, Subcommand};
use dirs;
use serde_json;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
};

/// Docker Credential Helpers 命令行工具
///
/// 用于管理 Docker 注册表的凭据，支持存储、检索和删除凭据
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 存储凭据
    Store,
    /// 检索凭据
    Get,
    /// 删除凭据
    Erase,
    /// 列出所有凭据
    List,
}

/// 凭据存储结构
struct CredentialStore {
    path: String,
}

impl CredentialStore {
    /// 创建新的凭据存储
    fn new() -> Self {
        let home_dir = dirs::home_dir().expect("无法获取主目录");
        let path = home_dir.join(".docker").join("config.json").to_string_lossy().to_string();
        Self { path }
    }

    /// 读取配置文件
    fn read_config(&self) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
        if !Path::new(&self.path).exists() {
            return Ok(HashMap::new());
        }

        let file = File::open(&self.path)?;
        let config: serde_json::Value = serde_json::from_reader(file)?;

        let empty_auths = serde_json::Value::Object(serde_json::Map::new());
        let auths = config.get("auths").unwrap_or(&empty_auths);

        let mut auth_map = HashMap::new();
        if let serde_json::Value::Object(auths_obj) = auths {
            for (registry, auth_info) in auths_obj {
                auth_map.insert(registry.clone(), auth_info.clone());
            }
        }

        Ok(auth_map)
    }

    /// 写入配置文件
    fn write_config(&self, auths: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        let config = serde_json::json!({
            "auths": auths
        });

        let config_dir = Path::new(&self.path).parent().unwrap();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let file = File::create(&self.path)?;
        serde_json::to_writer_pretty(file, &config)?;
        Ok(())
    }

    /// 存储凭据
    fn store(&self, registry: &str, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut auths = self.read_config()?;

        let auth = base64::encode(format!("{}:{}", username, password));
        auths.insert(
            registry.to_string(),
            serde_json::json!({
                "auth": auth
            }),
        );

        self.write_config(&auths)
    }

    /// 检索凭据
    fn get(&self, registry: &str) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
        let auths = self.read_config()?;

        if let Some(auth_info) = auths.get(registry) {
            if let Some(auth) = auth_info.get("auth").and_then(|a| a.as_str()) {
                let decoded = base64::decode(auth)?;
                let credentials = String::from_utf8(decoded)?;
                if let Some((username, password)) = credentials.split_once(":") {
                    return Ok(Some((username.to_string(), password.to_string())));
                }
            }
        }

        Ok(None)
    }

    /// 删除凭据
    fn erase(&self, registry: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut auths = self.read_config()?;
        auths.remove(registry);
        self.write_config(&auths)
    }

    /// 列出所有凭据
    fn list(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let auths = self.read_config()?;

        let mut result = HashMap::new();
        for (registry, _) in auths {
            result.insert(registry, "".to_string());
        }

        Ok(result)
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let store = CredentialStore::new();

    match cli.command {
        Commands::Store => {
            // 从标准输入读取 JSON 格式的凭据
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).expect("无法读取输入");

            let credential: serde_json::Value = serde_json::from_str(&input).expect("无效的 JSON 格式");

            let registry = credential.get("ServerURL").and_then(|v| v.as_str()).expect("缺少 ServerURL");
            let username = credential.get("Username").and_then(|v| v.as_str()).expect("缺少 Username");
            let password = credential.get("Secret").and_then(|v| v.as_str()).expect("缺少 Secret");

            if let Err(e) = store.store(registry, username, password) {
                eprintln!("存储凭据失败: {:?}", e);
                std::process::exit(1);
            }
        }
        Commands::Get => {
            // 从标准输入读取注册表地址
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).expect("无法读取输入");
            let registry = input.trim();

            match store.get(registry) {
                Ok(Some((username, password))) => {
                    let result = serde_json::json!({
                        "Username": username,
                        "Secret": password
                    });
                    println!("{}", serde_json::to_string(&result).unwrap());
                }
                Ok(None) => {
                    // 未找到凭据，输出空 JSON
                    println!("{{}}");
                }
                Err(e) => {
                    eprintln!("获取凭据失败: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Erase => {
            // 从标准输入读取注册表地址
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).expect("无法读取输入");
            let registry = input.trim();

            if let Err(e) = store.erase(registry) {
                eprintln!("删除凭据失败: {:?}", e);
                std::process::exit(1);
            }
        }
        Commands::List => match store.list() {
            Ok(registries) => {
                let result = serde_json::json!(registries);
                println!("{}", serde_json::to_string(&result).unwrap());
            }
            Err(e) => {
                eprintln!("列出凭据失败: {:?}", e);
                std::process::exit(1);
            }
        },
    }
}
