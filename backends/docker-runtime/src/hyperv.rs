use std::{process::Command, str};

/// Hyper-V 虚拟机管理器
#[derive(Debug)]
pub struct HyperVManager {
    /// 虚拟机名称前缀
    pub vm_prefix: String,
}

impl HyperVManager {
    /// 创建新的 Hyper-V 管理器
    pub fn new(vm_prefix: &str) -> Self {
        Self { vm_prefix: vm_prefix.to_string() }
    }

    /// 检查 Hyper-V 是否已启用
    pub fn is_hyperv_enabled(&self) -> bool {
        let output = Command::new("powershell")
            .args(&["-Command", "Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V"])
            .output()
            .expect("Failed to execute PowerShell command");

        let output_str = str::from_utf8(&output.stdout).unwrap();
        output_str.contains("State : Enabled")
    }

    /// 创建新的 Hyper-V 虚拟机
    pub fn create_vm(&self, vm_name: &str, memory_mb: u32, disk_size_gb: u32) -> Result<(), String> {
        let full_vm_name = format!("{}-{}", self.vm_prefix, vm_name);

        // 检查虚拟机是否已存在
        let check_output = Command::new("powershell")
            .args(&["-Command", &format!("Get-VM -Name '{}' -ErrorAction SilentlyContinue", full_vm_name)])
            .output()
            .expect("Failed to execute PowerShell command");

        let check_output_str = str::from_utf8(&check_output.stdout).unwrap();
        if !check_output_str.is_empty() {
            return Err(format!("Virtual machine '{}' already exists", full_vm_name));
        }

        // 创建虚拟机
        let create_output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "New-VM -Name '{}' -MemoryStartupBytes {}MB -NewVHDPath 'C:\\Hyper-V\\Virtual Hard Disks\\{}.vhdx' -NewVHDSizeBytes {}GB -Generation 2",
                    full_vm_name,
                    memory_mb,
                    full_vm_name,
                    disk_size_gb
                ),
            ])
            .output()
            .expect("Failed to execute PowerShell command");

        if !create_output.status.success() {
            let error_str = str::from_utf8(&create_output.stderr).unwrap();
            return Err(format!("Failed to create virtual machine: {}", error_str));
        }

        Ok(())
    }

    /// 启动虚拟机
    pub fn start_vm(&self, vm_name: &str) -> Result<(), String> {
        let full_vm_name = format!("{}-{}", self.vm_prefix, vm_name);

        let output = Command::new("powershell")
            .args(&["-Command", &format!("Start-VM -Name '{}'", full_vm_name)])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to start virtual machine: {}", error_str));
        }

        Ok(())
    }

    /// 停止虚拟机
    pub fn stop_vm(&self, vm_name: &str) -> Result<(), String> {
        let full_vm_name = format!("{}-{}", self.vm_prefix, vm_name);

        let output = Command::new("powershell")
            .args(&["-Command", &format!("Stop-VM -Name '{}' -Force", full_vm_name)])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to stop virtual machine: {}", error_str));
        }

        Ok(())
    }

    /// 删除虚拟机
    pub fn remove_vm(&self, vm_name: &str) -> Result<(), String> {
        let full_vm_name = format!("{}-{}", self.vm_prefix, vm_name);

        // 先停止虚拟机
        let _ = self.stop_vm(vm_name);

        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Remove-VM -Name '{}' -Force; Remove-Item -Path 'C:\\Hyper-V\\Virtual Hard Disks\\{}.vhdx' -Force -ErrorAction SilentlyContinue",
                    full_vm_name,
                    full_vm_name
                ),
            ])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to remove virtual machine: {}", error_str));
        }

        Ok(())
    }

    /// 列出所有虚拟机
    pub fn list_vms(&self) -> Result<Vec<String>, String> {
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Get-VM | Where-Object {{ $_.Name -like '{}*' }} | Select-Object -ExpandProperty Name",
                    self.vm_prefix
                ),
            ])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to list virtual machines: {}", error_str));
        }

        let output_str = str::from_utf8(&output.stdout).unwrap();
        let vms: Vec<String> = output_str.lines().filter(|line| !line.is_empty()).map(|line| line.trim().to_string()).collect();

        Ok(vms)
    }

    /// 获取虚拟机状态
    pub fn get_vm_status(&self, vm_name: &str) -> Result<String, String> {
        let full_vm_name = format!("{}-{}", self.vm_prefix, vm_name);

        let output = Command::new("powershell")
            .args(&["-Command", &format!("Get-VM -Name '{}' | Select-Object -ExpandProperty State", full_vm_name)])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to get virtual machine status: {}", error_str));
        }

        let output_str = str::from_utf8(&output.stdout).unwrap().trim().to_string();
        Ok(output_str)
    }
}
