
#[cfg(test)]
mod tests {
    use docker::cgroup::{CgroupManager, CgroupVersion};
    use docker_types::ResourceLimits;

    #[test]
    fn test_cgroup_version_enum() {
        let v1 = CgroupVersion::V1;
        let v2 = CgroupVersion::V2;

        assert_ne!(v1, v2);
        assert_eq!(v1, CgroupVersion::V1);
        assert_eq!(v2, CgroupVersion::V2);
    }

    #[test]
    fn test_cgroup_manager_default() {
        let _manager = CgroupManager::default();
    }

    #[test]
    fn test_cgroup_manager_new() {
        let manager = CgroupManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_resource_limits_construction() {
        let limits = ResourceLimits {
            cpu_limit: 1.0,
            memory_limit: 512,
            storage_limit: 10,
            network_limit: 100,
        };

        assert_eq!(limits.cpu_limit, 1.0);
        assert_eq!(limits.memory_limit, 512);
        assert_eq!(limits.storage_limit, 10);
        assert_eq!(limits.network_limit, 100);
    }
}
