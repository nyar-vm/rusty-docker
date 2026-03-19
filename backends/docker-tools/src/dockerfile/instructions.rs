//! Dockerfile instructions module
//! 
//! This module provides functionality to handle individual Dockerfile instructions.

use docker_types::DockerError;
use oak_dockerfile::ast::Instruction;
use std::fs;
use std::path::PathBuf;
use crate::dockerfile::context::BuildContext;

/// Handle a single Dockerfile instruction
///
/// # Parameters
/// * `instruction` - The instruction to handle
/// * `context` - The build context directory
///
/// # Returns
/// * `Result<(), DockerError>` - Handling result
pub fn handle_instruction(instruction: &Instruction, context: &std::path::Path) -> Result<(), DockerError> {
    match instruction {
        Instruction::From { image, tag, .. } => {
            handle_from(image, tag)?;
        },
        Instruction::Run { command, .. } => {
            handle_run(command)?;
        },
        Instruction::Copy { src, dest, .. } => {
            handle_copy(src, dest, context)?;
        },
        Instruction::Add { src, dest, .. } => {
            handle_add(src, dest, context)?;
        },
        Instruction::Workdir { path, .. } => {
            handle_workdir(path)?;
        },
        Instruction::Expose { port, .. } => {
            handle_expose(port)?;
        },
        Instruction::Env { key, value, .. } => {
            handle_env(key, value)?;
        },
        Instruction::Cmd { command, .. } => {
            handle_cmd(command)?;
        },
        Instruction::Entrypoint { command, .. } => {
            handle_entrypoint(command)?;
        },
        Instruction::Volume { path, .. } => {
            handle_volume(path)?;
        },
        Instruction::User { user, .. } => {
            handle_user(user)?;
        },
        Instruction::Label { key, value, .. } => {
            handle_label(key, value)?;
        },
        Instruction::Arg { name, default, .. } => {
            handle_arg(name, default)?;
        },
        Instruction::Onbuild { instruction, .. } => {
            handle_onbuild(instruction, context)?;
        },
        Instruction::Stopsignal { signal, .. } => {
            handle_stopsignal(signal)?;
        },
        Instruction::Healthcheck { command, .. } => {
            handle_healthcheck(command)?;
        },
        Instruction::Shell { shell, .. } => {
            handle_shell(shell)?;
        },
    }
    
    Ok(())
}

/// Handle FROM instruction
fn handle_from(image: &str, tag: &Option<String>) -> Result<(), DockerError> {
    println!("FROM: {}:{}", image, tag.as_deref().unwrap_or("latest"));
    Ok(())
}

/// Handle RUN instruction
fn handle_run(command: &str) -> Result<(), DockerError> {
    println!("RUN: {}", command);
    Ok(())
}

/// Handle COPY instruction
fn handle_copy(src: &str, dest: &str, context: &std::path::Path) -> Result<(), DockerError> {
    let build_context = BuildContext::new(context)?;
    let dest_path = PathBuf::from(dest);
    
    println!("COPY: {} -> {:?}", src, dest_path);
    
    // Check if source is a directory
    let src_path = build_context.get_absolute_path(src);
    if src_path.is_dir() {
        build_context.copy_directory(src, &dest_path)?;
    } else {
        build_context.copy_file(src, &dest_path)?;
    }
    
    Ok(())
}

/// Handle ADD instruction
fn handle_add(src: &str, dest: &str, context: &std::path::Path) -> Result<(), DockerError> {
    let build_context = BuildContext::new(context)?;
    let dest_path = PathBuf::from(dest);
    
    println!("ADD: {} -> {:?}", src, dest_path);
    
    // Check if source is a directory or archive
    let src_path = build_context.get_absolute_path(src);
    if src_path.is_dir() {
        build_context.copy_directory(src, &dest_path)?;
    } else {
        // For ADD, we would also handle archive extraction, but for now we just copy
        build_context.copy_file(src, &dest_path)?;
    }
    
    Ok(())
}

/// Handle WORKDIR instruction
fn handle_workdir(path: &str) -> Result<(), DockerError> {
    println!("WORKDIR: {}", path);
    Ok(())
}

/// Handle EXPOSE instruction
fn handle_expose(port: &str) -> Result<(), DockerError> {
    println!("EXPOSE: {}", port);
    Ok(())
}

/// Handle ENV instruction
fn handle_env(key: &str, value: &str) -> Result<(), DockerError> {
    println!("ENV: {}={}", key, value);
    Ok(())
}

/// Handle CMD instruction
fn handle_cmd(command: &str) -> Result<(), DockerError> {
    println!("CMD: {}", command);
    Ok(())
}

/// Handle ENTRYPOINT instruction
fn handle_entrypoint(command: &str) -> Result<(), DockerError> {
    println!("ENTRYPOINT: {}", command);
    Ok(())
}

/// Handle VOLUME instruction
fn handle_volume(path: &str) -> Result<(), DockerError> {
    println!("VOLUME: {}", path);
    Ok(())
}

/// Handle USER instruction
fn handle_user(user: &str) -> Result<(), DockerError> {
    println!("USER: {}", user);
    Ok(())
}

/// Handle LABEL instruction
fn handle_label(key: &str, value: &str) -> Result<(), DockerError> {
    println!("LABEL: {}={}", key, value);
    Ok(())
}

/// Handle ARG instruction
fn handle_arg(name: &str, default: &Option<String>) -> Result<(), DockerError> {
    if let Some(default) = default {
        println!("ARG: {}={}", name, default);
    } else {
        println!("ARG: {}", name);
    }
    Ok(())
}

/// Handle ONBUILD instruction
fn handle_onbuild(instruction: &Instruction, context: &std::path::Path) -> Result<(), DockerError> {
    println!("ONBUILD:");
    handle_instruction(instruction, context)?;
    Ok(())
}

/// Handle STOPSIGNAL instruction
fn handle_stopsignal(signal: &str) -> Result<(), DockerError> {
    println!("STOPSIGNAL: {}", signal);
    Ok(())
}

/// Handle HEALTHCHECK instruction
fn handle_healthcheck(command: &str) -> Result<(), DockerError> {
    println!("HEALTHCHECK: {}", command);
    Ok(())
}

/// Handle SHELL instruction
fn handle_shell(shell: &str) -> Result<(), DockerError> {
    println!("SHELL: {}", shell);
    Ok(())
}
