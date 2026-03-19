use docker_tools::dockerfile::{parser::parse_dockerfile, executor::execute_dockerfile};
use tempfile::tempdir;

#[test]
fn test_execute_dockerfile() {
    let dockerfile_content = r#"
FROM alpine:latest
WORKDIR /app
COPY . .
RUN echo "Hello, World!"
EXPOSE 8080
CMD ["echo", "Server started"]
"#;

    // Parse the Dockerfile
    let ast = parse_dockerfile(dockerfile_content).expect("Failed to parse Dockerfile");
    
    // Create a temporary directory as build context
    let temp_dir = tempdir().expect("Failed to create temp dir");
    
    // Execute the Dockerfile
    let result = execute_dockerfile(&ast, temp_dir.path());
    assert!(result.is_ok(), "Failed to execute Dockerfile: {:?}", result);
    
    let state = result.unwrap();
    
    // Verify the execution state
    assert_eq!(state.working_dir, "/app");
    assert_eq!(state.base_image, Some("alpine:latest".to_string()));
    assert!(state.exposed_ports.contains(&"8080".to_string()));
    assert_eq!(state.cmd, Some("[\"echo\", \"Server started\"]".to_string()));
}

#[test]
fn test_execute_dockerfile_with_env() {
    let dockerfile_content = r#"
FROM alpine:latest
ENV APP_NAME=myapp
ENV APP_VERSION=1.0.0
WORKDIR /app
"#;

    // Parse the Dockerfile
    let ast = parse_dockerfile(dockerfile_content).expect("Failed to parse Dockerfile");
    
    // Create a temporary directory as build context
    let temp_dir = tempdir().expect("Failed to create temp dir");
    
    // Execute the Dockerfile
    let result = execute_dockerfile(&ast, temp_dir.path());
    assert!(result.is_ok(), "Failed to execute Dockerfile: {:?}", result);
    
    let state = result.unwrap();
    
    // Verify environment variables
    assert_eq!(state.env_vars.get("APP_NAME"), Some(&"myapp".to_string()));
    assert_eq!(state.env_vars.get("APP_VERSION"), Some(&"1.0.0".to_string()));
}
