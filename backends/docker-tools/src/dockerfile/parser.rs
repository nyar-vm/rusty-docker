//! Dockerfile AST parser module
//! 
//! This module provides functionality to parse Dockerfiles into an AST using oak-dockerfile.

use docker_types::DockerError;
use oak_dockerfile::{DockerfileLanguage, DockerfileBuilder};
use oak_core::source::SourceText;
use oak_core::builder::NoBuilderCache;

/// Parse a Dockerfile into an AST
///
/// # Parameters
/// * `content` - The Dockerfile content as a string
///
/// # Returns
/// * `Result<oak_dockerfile::ast::DockerfileRoot, DockerError>` - The parsed AST or an error
pub fn parse_dockerfile(content: &str) -> Result<oak_dockerfile::ast::DockerfileRoot, DockerError> {
    // Create language configuration
    let language = DockerfileLanguage::default();
    let builder = DockerfileBuilder::new(&language);
    
    // Create source text from content
    let source_text = SourceText::new(content.to_string());
    let mut cache = NoBuilderCache;
    
    // Build AST
    let result = builder.build(&source_text, &[], &mut cache);
    
    result.result
        .map_err(|e| DockerError::container_error(format!("Failed to parse Dockerfile: {:?}", e)))
}

/// Parse a Dockerfile from a file
///
/// # Parameters
/// * `path` - The path to the Dockerfile
///
/// # Returns
/// * `Result<oak_dockerfile::ast::DockerfileRoot, DockerError>` - The parsed AST or an error
pub fn parse_dockerfile_from_file(path: &std::path::Path) -> Result<oak_dockerfile::ast::DockerfileRoot, DockerError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| DockerError::container_error(format!("Failed to read Dockerfile: {}", e)))?;
    
    parse_dockerfile(&content)
}