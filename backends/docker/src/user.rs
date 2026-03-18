use docker_types::{DockerError, Result as DockerResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// 用户角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    /// 管理员角色，拥有所有权限
    Admin,
    /// 普通用户角色，拥有基本操作权限
    User,
    /// 只读角色，只能查看信息
    ReadOnly,
}

/// 权限类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    /// 管理用户
    ManageUsers,
    /// 管理容器
    ManageContainers,
    /// 管理镜像
    ManageImages,
    /// 管理网络
    ManageNetworks,
    /// 管理卷
    ManageVolumes,
    /// 管理服务
    ManageServices,
    /// 管理节点
    ManageNodes,
    /// 管理堆栈
    ManageStacks,
    /// 查看系统状态
    ViewSystemStatus,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户ID
    pub id: String,
    /// 用户名
    pub username: String,
    /// 密码哈希
    pub password_hash: String,
    /// 角色
    pub role: Role,
    /// 创建时间
    pub created_at: std::time::SystemTime,
    /// 最后登录时间
    pub last_login: Option<std::time::SystemTime>,
}

/// 角色权限映射
pub fn get_role_permissions(role: &Role) -> Vec<Permission> {
    match role {
        Role::Admin => {
            vec![
                Permission::ManageUsers,
                Permission::ManageContainers,
                Permission::ManageImages,
                Permission::ManageNetworks,
                Permission::ManageVolumes,
                Permission::ManageServices,
                Permission::ManageNodes,
                Permission::ManageStacks,
                Permission::ViewSystemStatus,
            ]
        }
        Role::User => {
            vec![
                Permission::ManageContainers,
                Permission::ManageImages,
                Permission::ManageNetworks,
                Permission::ManageVolumes,
                Permission::ViewSystemStatus,
            ]
        }
        Role::ReadOnly => {
            vec![Permission::ViewSystemStatus]
        }
    }
}

/// 用户管理服务
pub struct UserManager {
    /// 用户存储
    users: Arc<Mutex<HashMap<String, User>>>,
}

impl UserManager {
    /// 创建新的用户管理服务
    pub fn new() -> Self {
        let users = Arc::new(Mutex::new(HashMap::new()));

        // 创建默认管理员用户
        let admin_user = User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password_hash: "admin".to_string(), // 实际应用中应该使用密码哈希
            role: Role::Admin,
            created_at: std::time::SystemTime::now(),
            last_login: None,
        };

        users.lock().unwrap().insert(admin_user.id.clone(), admin_user);

        Self { users }
    }

    /// 创建用户
    pub fn create_user(&self, username: String, password: String, role: Role) -> DockerResult<User> {
        let mut users = self.users.lock().map_err(|e| DockerError::io_error("lock_error", e.to_string()))?;

        // 检查用户名是否已存在
        for user in users.values() {
            if user.username == username {
                return Err(DockerError::container_error("用户名已存在".to_string()));
            }
        }

        let user = User {
            id: Uuid::new_v4().to_string(),
            username,
            password_hash: password, // 实际应用中应该使用密码哈希
            role,
            created_at: std::time::SystemTime::now(),
            last_login: None,
        };

        users.insert(user.id.clone(), user.clone());
        Ok(user)
    }

    /// 获取用户
    pub fn get_user(&self, user_id: &str) -> DockerResult<User> {
        let users = self.users.lock().map_err(|e| DockerError::io_error("lock_error", e.to_string()))?;

        users.get(user_id).cloned().ok_or_else(|| DockerError::container_error("用户不存在".to_string()))
    }

    /// 获取用户通过用户名
    pub fn get_user_by_username(&self, username: &str) -> DockerResult<User> {
        let users = self.users.lock().map_err(|e| DockerError::io_error("lock_error", e.to_string()))?;

        users
            .values()
            .find(|user| user.username == username)
            .cloned()
            .ok_or_else(|| DockerError::container_error("用户不存在".to_string()))
    }

    /// 列出所有用户
    pub fn list_users(&self) -> DockerResult<Vec<User>> {
        let users = self.users.lock().map_err(|e| DockerError::io_error("lock_error", e.to_string()))?;

        Ok(users.values().cloned().collect())
    }

    /// 更新用户
    pub fn update_user(
        &self,
        user_id: &str,
        username: Option<String>,
        password: Option<String>,
        role: Option<Role>,
    ) -> DockerResult<User> {
        let mut users = self.users.lock().map_err(|e| DockerError::io_error("lock_error", e.to_string()))?;

        // 先检查用户是否存在
        if !users.contains_key(user_id) {
            return Err(DockerError::container_error("用户不存在".to_string()));
        }

        let mut new_username = None;

        // 检查用户名是否已被其他用户使用
        if let Some(username) = username {
            for (id, other_user) in users.iter() {
                if other_user.username == username && id != user_id {
                    return Err(DockerError::container_error("用户名已存在".to_string()));
                }
            }
            new_username = Some(username);
        }

        let user = users.get_mut(user_id).unwrap();

        if let Some(username) = new_username {
            user.username = username;
        }

        if let Some(password) = password {
            user.password_hash = password; // 实际应用中应该使用密码哈希
        }

        if let Some(role) = role {
            user.role = role;
        }

        Ok(user.clone())
    }

    /// 删除用户
    pub fn delete_user(&self, user_id: &str) -> DockerResult<()> {
        let mut users = self.users.lock().map_err(|e| DockerError::io_error("lock_error", e.to_string()))?;

        if !users.contains_key(user_id) {
            return Err(DockerError::container_error("用户不存在".to_string()));
        }

        // 不允许删除最后一个管理员用户
        let admin_count = users.values().filter(|user| user.role == Role::Admin).count();

        if admin_count == 1 && users.get(user_id).unwrap().role == Role::Admin {
            return Err(DockerError::container_error("不能删除最后一个管理员用户".to_string()));
        }

        users.remove(user_id);
        Ok(())
    }

    /// 验证用户凭据
    pub fn authenticate(&self, username: &str, password: &str) -> DockerResult<User> {
        let mut users = self.users.lock().map_err(|e| DockerError::io_error("lock_error", e.to_string()))?;

        let user = users
            .values_mut()
            .find(|user| user.username == username && user.password_hash == password)
            .ok_or_else(|| DockerError::container_error("用户名或密码错误".to_string()))?;

        // 更新最后登录时间
        user.last_login = Some(std::time::SystemTime::now());

        Ok(user.clone())
    }

    /// 检查用户是否有指定权限
    pub fn check_permission(&self, user_id: &str, permission: &Permission) -> DockerResult<bool> {
        let user = self.get_user(user_id)?;
        let permissions = get_role_permissions(&user.role);
        Ok(permissions.contains(permission))
    }

    /// 获取用户角色
    pub fn get_user_role(&self, user_id: &str) -> DockerResult<Role> {
        let user = self.get_user(user_id)?;
        Ok(user.role)
    }

    /// 更新用户角色
    pub fn update_user_role(&self, user_id: &str, role: Role) -> DockerResult<User> {
        let mut users = self.users.lock().map_err(|e| DockerError::io_error("lock_error", e.to_string()))?;

        // 先检查用户是否存在
        if !users.contains_key(user_id) {
            return Err(DockerError::container_error("用户不存在".to_string()));
        }

        let current_user = users.get(user_id).unwrap();

        // 不允许将最后一个管理员用户的角色修改为非管理员
        if current_user.role == Role::Admin {
            let admin_count = users.values().filter(|u| u.role == Role::Admin).count();

            if admin_count == 1 && role != Role::Admin {
                return Err(DockerError::container_error("不能将最后一个管理员用户的角色修改为非管理员".to_string()));
            }
        }

        let user = users.get_mut(user_id).unwrap();
        user.role = role;
        Ok(user.clone())
    }
}
