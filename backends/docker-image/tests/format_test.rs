use docker_image2::format::*;

#[test]
fn test_parse_image_reference() {
    let cases = [
        ("ubuntu", ("docker.io", "ubuntu", "latest")),
        ("ubuntu:latest", ("docker.io", "ubuntu", "latest")),
        ("docker.io/ubuntu", ("docker.io", "ubuntu", "latest")),
        ("docker.io/ubuntu:20.04", ("docker.io", "ubuntu", "20.04")),
        ("localhost:5000/ubuntu", ("localhost:5000", "ubuntu", "latest")),
    ];

    for (input, expected) in cases {
        let (registry, repo, tag) = parse_image_reference(input).unwrap();
        assert_eq!(registry, expected.0);
        assert_eq!(repo, expected.1);
        assert_eq!(tag, expected.2);
    }
}
