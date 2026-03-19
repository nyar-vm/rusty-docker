//! Dockerfile build context management module
//! 
//! This module provides functionality to manage the build context for Dockerfile execution.

use docker_types::DockerError;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

/// Build context manager
#[derive(Debug)]
pub struct BuildContext {
    /// Root directory of the build context
    root: PathBuf,
    /// Ignored paths based on .dockerignore
    ignored_paths: HashSet<PathBuf>,
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
        
        let ignored_paths = Self::read_dockerignore(root)?;
        
        Ok(Self {
            root: root.to_path_buf(),
            ignored_paths,
        })
    }
    
    /// Read .dockerignore file and return ignored paths
    ///
    /// # Parameters
    /// * `root` - The root directory of the build context
    ///
    /// # Returns
    /// * `Result<HashSet<PathBuf>, DockerError>` - The set of ignored paths
    fn read_dockerignore(root: &Path) -> Result<HashSet<PathBuf>, DockerError> {
        let dockerignore_path = root.join(".dockerignore");
        if !dockerignore_path.exists() {
            return Ok(HashSet::new());
        }
        
        let content = fs::read_to_string(&dockerignore_path)
            .map_err(|e| DockerError::container_error(format!("Failed to read .dockerignore: {}", e)))?;
        
        let mut ignored = HashSet::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Convert the pattern to a PathBuf
            let pattern = Path::new(line);
            let absolute_path = root.join(pattern);
            ignored.insert(absolute_path);
        }
        
        Ok(ignored)
    }
    
    /// Check if a path is ignored
    ///
    /// # Parameters
    /// * `path` - The path to check
    ///
    /// # Returns
    /// * `bool` - Whether the path is ignored
    pub fn is_ignored(&self, path: &Path) -> bool {
        // Check if the path itself is ignored
        if self.ignored_paths.contains(path) {
            return true;
        }
        
        // Check if any parent directory is ignored
        let mut current = path;
        while let Some(parent) = current.parent() {
            if self.ignored_paths.contains(parent) {
                return true;
            }
            current = parent;
        }
        
        false
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