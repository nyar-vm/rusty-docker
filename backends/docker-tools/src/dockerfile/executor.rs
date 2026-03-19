//! Dockerfile AST executor module
//! 
//! This module provides functionality to execute Dockerfile instructions using AST.

use docker_types::DockerError;
use oak_dockerfile::ast::{DockerfileRoot, Instruction};
use crate::dockerfile::instructions::handle_instruction;

/// Execution state for Dockerfile instructions
#[derive(Debug, Default)]
pub struct ExecutionState {
    /// Current working directory
    pub working_dir: String,
    /// Environment variables
    pub env_vars: std::collections::HashMap<String, String>,
    /// Current base image
    pub base_image: Option<String>,
    /// Exposed ports
    pub exposed_ports: Vec<String>,
    /// Entrypoint command
    pub entrypoint: Option<String>,
    /// Command to run
    pub cmd: Option<String>,
}

/// Execute Dockerfile instructions from AST
///
/// # Parameters
/// * `ast` - The parsed Dockerfile AST
/// * `context` - The build context directory
///
/// # Returns
/// * `Result<ExecutionState, DockerError>` - Execution result with final state
pub fn execute_dockerfile(ast: &DockerfileRoot, context: &std::path::Path) -> Result<ExecutionState, DockerError> {
    let mut state = ExecutionState::default();
    
    // Set default working directory
    state.working_dir = "/".to_string();
    
    // Walk through each instruction and execute it
    for instruction in &ast.instructions {
        handle_instruction(instruction, context)?;
        
        // Update execution state based on instruction
        update_state(&mut state, instruction)?;
    }
    
    Ok(state)
}

/// Update execution state based on the instruction
///
/// # Parameters
/// * `state` - The current execution state
/// * `instruction` - The instruction that was executed
///
/// # Returns
/// * `Result<(), DockerError>` - Update result
fn update_state(state: &mut ExecutionState, instruction: &Instruction) -> Result<(), DockerError> {
    match instruction {
        Instruction::From { image, tag, .. } => {
            let full_image = if let Some(tag) = tag {
                format!("{}:{}", image, tag)
            } else {
                image.to_string()
            };
            state.base_image = Some(full_image);
        },
        Instruction::Workdir { path, .. } => {
            state.working_dir = path.to_string();
        },
        Instruction::Env { key, value, .. } => {
            state.env_vars.insert(key.to_string(), value.to_string());
        },
        Instruction::Expose { port, .. } => {
            state.exposed_ports.push(port.to_string());
        },
        Instruction::Entrypoint { command, .. } => {
            state.entrypoint = Some(command.to_string());
        },
        Instruction::Cmd { command, .. } => {
            state.cmd = Some(command.to_string());
        },
        _ => {
            // Other instructions don't affect the state
        }
    }
    
    Ok(())
}
