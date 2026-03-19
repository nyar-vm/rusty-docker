use docker_tools::dockerfile::context::BuildContext;
use tempfile::tempdir;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_build_context_creation() {
    // Create a temporary directory as build context
    let temp_dir = tempdir().expect("Failed to create temp dir");
    
    // Create a test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").expect("Failed to write test file");
    
    // Create build context
    let result = BuildContext::new(temp_dir.path());
    assert!(result.is_ok(), "Failed to create build context: {:?}", result);
    
    let context = result.unwrap();
    assert_eq!(context.get_absolute_path("test.txt"), test_file);
}

#[test]
fn test_dockerignore() {
    // Create a temporary directory as build context
    let temp_dir = tempdir().expect("Failed to create temp dir");
    
    // Create .dockerignore file
    let dockerignore_path = temp_dir.path().join(".dockerignore");
    fs::write(&dockerignore_path, "ignored.txt\n*.log").expect("Failed to write .dockerignore");
    
    // Create test files
    let test_file = temp_dir.path().join("test.txt");
    let ignored_file = temp_dir.path().join("ignored.txt");
    let log_file = temp_dir.path().join("app.log");
    
    fs::write(&test_file, "Hello, World!").expect("Failed to write test file");
    fs::write(&ignored_file, "Ignored content").expect("Failed to write ignored file");
    fs::write(&log_file, "Log content").expect("Failed to write log file");
    
    // Create build context
    let context = BuildContext::new(temp_dir.path()).expect("Failed to create build context");
    
    // Check if files are ignored
    assert!(!context.is_ignored(&test_file), "test.txt should not be ignored");
    assert!(context.is_ignored(&ignored_file), "ignored.txt should be ignored");
    assert!(context.is_ignored(&log_file), "app.log should be ignored");
}

#[test]
fn test_copy_file() {
    // Create a temporary directory as build context
    let temp_dir = tempdir().expect("Failed to create temp dir");
    
    // Create a test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").expect("Failed to write test file");
    
    // Create build context
    let context = BuildContext::new(temp_dir.path()).expect("Failed to create build context");
    
    // Create a temporary destination directory
    let dest_dir = tempdir().expect("Failed to create destination dir");
    let dest_file = dest_dir.path().join("test.txt");
    
    // Copy the file
    let result = context.copy_file("test.txt", &dest_file);
    assert!(result.is_ok(), "Failed to copy file: {:?}", result);
    
    // Verify the file was copied
    assert!(dest_file.exists(), "Destination file does not exist");
    let content = fs::read_to_string(&dest_file).expect("Failed to read destination file");
    assert_eq!(content, "Hello, World!");
}
