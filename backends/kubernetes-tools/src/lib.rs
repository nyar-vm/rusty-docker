//! Kubernetes 工具集库
//!
//! 提供 Kubernetes 相关工具的共享功能和类型

#![warn(missing_docs)]

use clap::Command;

/// 创建基础命令配置
///
/// # 参数
/// * `name` - 命令名称
/// * `about` - 命令描述
///
/// # 返回值
/// 返回配置好的 Command 对象
pub fn create_base_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name)
        .about(about)
        .version("0.1.0")
        .author("Rusty Docker Team")
}

/// 运行 Kubernetes 命令
///
/// # 参数
/// * `cmd` - 命令对象
/// * `action` - 命令执行逻辑
///
/// # 返回值
/// 返回执行结果
pub fn run_command<F, E>(cmd: Command, action: F)
where
    F: FnOnce() -> Result<(), E>,
    E: std::fmt::Display,
{
    let _matches = cmd.get_matches();

    if let Err(e) = action() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
