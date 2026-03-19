use docker_tools::dockerfile::parser::{parse_dockerfile, parse_dockerfile_from_file};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_parse_valid_dockerfile() {
    let dockerfile_content = r#"
FROM alpine:latest
WORKDIR /app
COPY . .
RUN echo "Hello, World!"
EXPOSE 8080
CMD ["echo", "Server started"]
"#;

    let result = parse_dockerfile(dockerfile_content);
    assert!(result.is_ok(), "Failed to parse valid Dockerfile: {:?}", result);
    
    let ast = result.unwrap();
    assert!(!ast.instructions.is_empty(), "AST should contain instructions");
}

#[test]
fn test_parse_invalid_dockerfile() {
    let dockerfile_content = r#"
FROM alpine:latest
WORKDIR /app
COPY . .
RUN echo "Hello, World!"
INVALID_INSTRUCTION
EXPOSE 8080
"#;

    let result = parse_dockerfile(dockerfile_content);
    assert!(result.is_err(), "Should fail to parse invalid Dockerfile");
}

#[test]
fn test_parse_dockerfile_from_file() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    let mut file = File::create(&dockerfile_path).expect("Failed to create Dockerfile");
    writeln!(file, "FROM alpine:latest").expect("Failed to write to Dockerfile");
    writeln!(file, "WORKDIR /app").expect("Failed to write to Dockerfile");
    
    let result = parse_dockerfile_from_file(&dockerfile_path);
    assert!(result.is_ok(), "Failed to parse Dockerfile from file: {:?}", result);
    
    let ast = result.unwrap();
    assert!(!ast.instructions.is_empty(), "AST should contain instructions");
}
