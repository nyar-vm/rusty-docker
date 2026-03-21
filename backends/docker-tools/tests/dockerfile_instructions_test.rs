use docker_tools::dockerfile::{executor::execute_dockerfile, parser::parse_dockerfile};
use tempfile::tempdir;

#[test]
fn test_all_instructions() {
    let dockerfile_content = r#"
FROM alpine:latest

# Set environment variables
ENV APP_NAME=myapp
ENV APP_VERSION=1.0.0

# Set working directory
WORKDIR /app

# Copy files
COPY . .

# Run commands
RUN echo "Building application..."
RUN echo "APP_NAME=$APP_NAME" > app.txt

# Expose port
EXPOSE 8080

# Set entrypoint and command
ENTRYPOINT ["/bin/sh"]
CMD ["-c", "echo 'Server started on port 8080'"]

# Set user
USER root

# Add label
LABEL maintainer="test@example.com"

# Define build argument
ARG BUILD_ENV=production

# Set volume
VOLUME ["/data"]

# Set stop signal
STOPSIGNAL SIGTERM

# Set health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ || exit 1

# Set shell
SHELL ["/bin/sh", "-c"]

# Set onbuild instruction
ONBUILD RUN echo "This is an onbuild instruction"
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
    assert_eq!(state.entrypoint, Some("[/bin/sh]".to_string()));
    assert_eq!(state.cmd, Some("[-c, echo 'Server started on port 8080']".to_string()));
    assert_eq!(state.env_vars.get("APP_NAME"), Some(&"myapp".to_string()));
    assert_eq!(state.env_vars.get("APP_VERSION"), Some(&"1.0.0".to_string()));
}
