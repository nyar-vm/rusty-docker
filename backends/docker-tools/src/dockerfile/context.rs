//! Dockerfile build context management module
//! 
//! This module provides functionality to manage the build context for Dockerfile execution.

use docker_types::DockerError;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use regex;

/// Build context manager
#[derive(Debug)]
pub struct BuildContext {
    /// Root directory of the build context
    root: PathBuf,
    /// Ignored patterns based on .dockerignore
    ignored_patterns: Vec<String>,
}

impl BuildContext {
    /// Create a new build context from a directory
    ///
    /// # Parameters
    /// * `root` - The root directory of the build context
    ///
    /// # Returns
    /// * `Result<Self, DockerError>` - The build context or an error
    pub fn new(root: &Path) -> Result<Self, DockerError> {
        if !root.exists() || !root.is_dir() {
            return Err(DockerError::container_error(format!("Build context directory not found: {:?}", root)));
        }
        
        let ignored_patterns = Self::read_dockerignore(root)?;
        
        Ok(Self {
            root: root.to_path_buf(),
            ignored_patterns,
        })
    }
    
    /// Read .dockerignore file and return ignored patterns
    ///
    /// # Parameters
    /// * `root` - The root directory of the build context
    ///
    /// # Returns
    /// * `Result<Vec<String>, DockerError>` - The list of ignored patterns
    fn read_dockerignore(root: &Path) -> Result<Vec<String>, DockerError> {
        let dockerignore_path = root.join(".dockerignore");
        if !dockerignore_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&dockerignore_path)
            .map_err(|e| DockerError::container_error(format!("Failed to read .dockerignore: {}", e)))?;
        
        let mut patterns = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            patterns.push(line.to_string());
        }
        
        Ok(patterns)
    }
    
    /// Check if a path is ignored
    ///
    /// # Parameters
    /// * `path` - The path to check
    ///
    /// # Returns
    /// * `bool` - Whether the path is ignored
    pub fn is_ignored(&self, path: &Path) -> bool {
        // Get the relative path from the build context root
        let relative_path = match path.strip_prefix(&self.root) {
            Ok(path) => path,
            Err(_) => return false, // Path is outside the build context, not ignored
        };
        
        let relative_path_str = relative_path.to_string_lossy();
        
        // Check if the path matches any ignored pattern
        for pattern in &self.ignored_patterns {
            if self.matches_pattern(&relative_path_str, pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if a path matches a pattern
    ///
    /// # Parameters
    /// * `path` - The path to check
    /// * `pattern` - The pattern to match
    ///
    /// # Returns
    /// * `bool` - Whether the path matches the pattern
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob pattern matching
        // This is a basic implementation, real Dockerfile .dockerignore uses more complex rules
        let pattern = pattern.replace("*", ".*");
        let regex = regex::Regex::new(&format!("^{}$", pattern)).unwrap();
        regex.is_match(path)
    }
    
    /// Get the absolute path of a file in the build context
    ///
    /// # Parameters
    /// * `path` - The relative path within the build context
    ///
    /// # Returns
    /// * `PathBuf` - The absolute path
    pub fn get_absolute_path(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }
    
    /// Copy a file from the build context to a destination
    ///
    /// # Parameters
    /// * `src` - The source path within the build context
    /// * `dest` - The destination path
    ///
    /// # Returns
    /// * `Result<(), DockerError>` - Copy result
    pub fn copy_file(&self, src: &str, dest: &Path) -> Result<(), DockerError> {
        let src_path = self.get_absolute_path(src);
        
        // Check if the source path is ignored
        if self.is_ignored(&src_path) {
            return Err(DockerError::container_error(format!("Source path is ignored: {:?}", src)));
        }
        
        // Check if the source file exists
        if !src_path.exists() || !src_path.is_file() {
            return Err(DockerError::container_error(format!("Source file not found: {:?}", src)));
        }
        
        // Create the destination directory if it doesn't exist
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DockerError::container_error(format!("Failed to create destination directory: {}", e)))?;
        }
        
        // Copy the file
        fs::copy(&src_path, dest)
            .map_err(|e| DockerError::container_error(format!("Failed to copy file: {}", e)))?;
        
        Ok(())
    }
    
    /// Copy a directory from the build context to a destination
    ///
    /// # Parameters
    /// * `src` - The source directory within the build context
    /// * `dest` - The destination directory
    ///
    /// # Returns
    /// * `Result<(), DockerError>` - Copy result
    pub fn copy_directory(&self, src: &str, dest: &Path) -> Result<(), DockerError> {
        let src_path = self.get_absolute_path(src);
        
        // Check if the source path is ignored
        if self.is_ignored(&src_path) {
            return Err(DockerError::container_error(format!("Source directory is ignored: {:?}", src)));
        }
        
        // Check if the source directory exists
        if !src_path.exists() || !src_path.is_dir() {
            return Err(DockerError::container_error(format!("Source directory not found: {:?}", src)));
        }
        
        // Create the destination directory if it doesn't exist
        fs::create_dir_all(dest)
            .map_err(|e| DockerError::container_error(format!("Failed to create destination directory: {}", e)))?;
        
        // Copy all files in the directory
        for entry in fs::read_dir(&src_path)
            .map_err(|e| DockerError::container_error(format!("Failed to read source directory: {}", e)))? {
            let entry = entry
                .map_err(|e| DockerError::container_error(format!("Failed to read directory entry: {}", e)))?;
            let entry_path = entry.path();
            
            // Skip ignored paths
            if self.is_ignored(&entry_path) {
                continue;
            }
            
            let relative_path = entry_path.strip_prefix(&self.root)
                .map_err(|e| DockerError::container_error(format!("Failed to get relative path: {}", e)))?;
            let dest_path = dest.join(relative_path);
            
            if entry_path.is_dir() {
                self.copy_directory(relative_path.to_str().unwrap(), &dest_path)?;
            } else {
                self.copy_file(relative_path.to_str().unwrap(), &dest_path)?;
            }
        }
        
        Ok(())
    }
}