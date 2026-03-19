use std::{process::Command, str};

/// WSL 2 管理器
#[derive(Debug)]
pub struct Wsl2Manager {
    /// WSL 发行版名称
    pub distro_name: String,
}

impl Wsl2Manager {
    /// 创建新的 WSL 2 管理器
    pub fn new(distro_name: &str) -> Self {
        Self { distro_name: distro_name.to_string() }
    }

    /// 检查 WSL 2 是否已安装
    pub fn is_wsl2_installed(&self) -> bool {
        let output = Command::new("wsl").args(&["--version"]).output().expect("Failed to execute wsl command");

        output.status.success()
    }

    /// 检查指定的 WSL 发行版是否存在
    pub fn is_distro_exists(&self) -> bool {
        let output = Command::new("wsl").args(&["-l", "-v"]).output().expect("Failed to execute wsl command");

        let output_str = str::from_utf8(&output.stdout).unwrap();
        output_str.contains(&self.distro_name)
    }

    /// 创建新的 WSL 发行版
    pub fn create_distro(&self) -> Result<(), String> {
        // 检查发行版是否已存在
        if self.is_distro_exists() {
            return Ok(());
        }

        // 创建发行版
        let output = Command::new("wsl")
            .args(&["--install", "Ubuntu", "--distribution", &self.distro_name])
            .output()
            .expect("Failed to execute wsl command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to create WSL distro: {}", error_str));
        }

        Ok(())
    }

    /// 启动 WSL 发行版
    pub fn start_distro(&self) -> Result<(), String> {
        let output = Command::new("wsl")
            .args(&["-d", &self.distro_name, "echo", "WSL distro started"])
            .output()
            .expect("Failed to execute wsl command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to start WSL distro: {}", error_str));
        }

        Ok(())
    }

    /// 停止 WSL 发行版
    pub fn stop_distro(&self) -> Result<(), String> {
        let output =
            Command::new("wsl").args(&["--terminate", &self.distro_name]).output().expect("Failed to execute wsl command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to stop WSL distro: {}", error_str));
        }

        Ok(())
    }

    /// 执行 WSL 命令
    pub fn exec_command(&self, command: &str) -> Result<String, String> {
        let output = Command::new("wsl")
            .args(&["-d", &self.distro_name, "bash", "-c", command])
            .output()
            .expect("Failed to execute wsl command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to execute command: {}", error_str));
        }

        let output_str = str::from_utf8(&output.stdout).unwrap().trim().to_string();
        Ok(output_str)
    }

    /// 安装 Docker 在 WSL 发行版中
    pub fn install_docker(&self) -> Result<(), String> {
        // 执行 Docker 安装脚本
        let command = r#"curl -fsSL https://get.docker.com -o get-docker.sh && sh get-docker.sh && rm get-docker.sh"#;
        match self.exec_command(command) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to install Docker: {}", e)),
        }

        // 启动 Docker 服务
        let command = "sudo service docker start";
        match self.exec_command(command) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to start Docker service: {}", e)),
        }

        Ok(())
    }
}
