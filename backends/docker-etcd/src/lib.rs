//! Docker etcd
//!
//! Rust implementation of etcd server, providing distributed key-value storage

#![warn(missing_docs)]

use docker_types::DockerError;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast;

/// 结果类型
pub type Result<T> = std::result::Result<T, DockerError>;

/// 键值对
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyValue {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub create_revision: i64,
    pub mod_revision: i64,
    pub version: i64,
    pub lease: i64,
}

/// 存储引擎
pub struct Storage {
    data: RwLock<HashMap<Vec<u8>, KeyValue>>,
    revision: RwLock<i64>,
    tx: broadcast::Sender<Event>,
}

/// 事件类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventType {
    Put,
    Delete,
}

/// 事件
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub r#type: EventType,
    pub kv: KeyValue,
    pub prev_kv: Option<KeyValue>,
}

impl Storage {
    /// 创建新的存储引擎
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { data: RwLock::new(HashMap::new()), revision: RwLock::new(1), tx }
    }

    /// 获取值
    pub fn get(&self, key: &[u8]) -> Result<KeyValue> {
        let data = self.data.read().unwrap();
        data.get(key).cloned().ok_or(DockerError::not_found("key", "not found"))
    }

    /// 设置值
    pub fn put(&self, key: Vec<u8>, value: Vec<u8>, lease: i64) -> Result<KeyValue> {
        let mut data = self.data.write().unwrap();
        let mut revision = self.revision.write().unwrap();

        *revision += 1;
        let new_revision = *revision;

        let prev_kv = data.get(&key).cloned();

        let kv = KeyValue {
            key: key.clone(),
            value,
            create_revision: prev_kv.as_ref().map(|kv| kv.create_revision).unwrap_or(new_revision),
            mod_revision: new_revision,
            version: prev_kv.as_ref().map(|kv| kv.version + 1).unwrap_or(1),
            lease,
        };

        data.insert(key, kv.clone());

        // 发送事件
        let event = Event { r#type: EventType::Put, kv: kv.clone(), prev_kv };
        self.tx.send(event).ok();

        Ok(kv)
    }

    /// 删除值
    pub fn delete(&self, key: &[u8]) -> Result<Option<KeyValue>> {
        let mut data = self.data.write().unwrap();
        let prev_kv = data.remove(key);

        if let Some(kv) = &prev_kv {
            let mut revision = self.revision.write().unwrap();
            *revision += 1;

            // 发送事件
            let event = Event { r#type: EventType::Delete, kv: kv.clone(), prev_kv: None };
            self.tx.send(event).ok();
        }

        Ok(prev_kv)
    }

    /// 列出所有键
    pub fn list(&self) -> Vec<KeyValue> {
        let data = self.data.read().unwrap();
        data.values().cloned().collect()
    }

    /// 获取事件订阅
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    /// 获取当前版本
    pub fn get_revision(&self) -> i64 {
        *self.revision.read().unwrap()
    }
}

/// Etcd服务器
pub struct EtcdServer {
    storage: Arc<Storage>,
    address: String,
}

impl EtcdServer {
    /// 创建新的etcd服务器
    pub fn new(address: String) -> Self {
        Self { storage: Arc::new(Storage::new()), address }
    }

    /// 获取存储引擎
    pub fn storage(&self) -> Arc<Storage> {
        self.storage.clone()
    }

    /// 启动服务器
    pub async fn start(&self) -> Result<()> {
        // 这里将实现HTTP/gRPC服务器
        println!("Starting etcd server on {}", self.address);
        Ok(())
    }
}
