// Binary diff recording tests
// This module tests the binary diff recording functionality

#[cfg(test)]
mod tests {
    use rust_ecs::ecs::diff::*;
    use rust_ecs::ecs::replay::*;
    
    #[test]
    fn test_binary_diff_config() {
        // Test that binary configuration is created correctly
        let config = ReplayLogConfig::binary_optimized();
        assert!(config.binary_format);
        assert_eq!(config.log_directory, "binary_logs");
        assert_eq!(config.file_prefix, "binary_replay");
        assert!(config.include_component_details);
        assert_eq!(config.max_buffer_size, 4 * 1024 * 1024);
    }

    #[test]
    fn test_optimized_performance_config() {
        // Test that optimized performance uses binary format
        let config = ReplayLogConfig::optimized_performance();
        assert!(config.binary_format);
        assert!(config.minimal_mode);
        assert!(!config.include_component_details);
    }

    #[test]
    fn test_debug_config() {
        // Test that debug config uses text format for readability
        let config = ReplayLogConfig::debug_full();
        assert!(!config.binary_format);
        assert!(!config.minimal_mode);
        assert!(config.include_component_details);
    }

    #[test]
    fn test_default_config() {
        // Test that default config uses text format for compatibility
        let config = ReplayLogConfig::default();
        assert!(!config.binary_format);
        assert!(!config.enabled);
    }

    #[test]
    fn test_binary_diff_primitive_types() {
        // Test binary serialization for primitive types
        let value1: u32 = 42;
        let value2: u32 = 84;
        
        if let Some(diff) = value1.diff(&value2) {
            // Test binary serialization
            match u32::diff_to_binary(&diff) {
                Ok(binary_data) => {
                    assert!(!binary_data.is_empty());
                    
                    // Test deserialization
                    match u32::diff_from_binary(&binary_data) {
                        Ok(restored_diff) => {
                            assert_eq!(diff, restored_diff);
                        }
                        Err(e) => panic!("Failed to deserialize u32 diff: {}", e),
                    }
                }
                Err(e) => panic!("Failed to serialize u32 diff: {}", e),
            }
        }
    }

    #[test]
    fn test_binary_diff_component_change() {
        use rust_ecs::ecs::core::Entity;
        
        // Test binary diff component change creation
        let entity = Entity::new(42, 0);
        let type_name = "TestComponent".to_string();
        let diff_data = vec![1, 2, 3, 4];
        
        let binary_change = BinaryDiffComponentChange::from_diff_change_raw(
            entity,
            type_name.clone(),
            diff_data.clone(),
            DiffChangeType::Modified,
        );
        
        match &binary_change {
            BinaryDiffComponentChange::Modified { entity: e, type_name: tn, diff_data: dd } => {
                assert_eq!(*e, entity);
                assert_eq!(tn, &type_name);
                assert_eq!(dd, &diff_data);
            }
            _ => panic!("Expected Modified variant"),
        }
        
        // Test conversion to text format
        let text_change = binary_change.to_diff_change();
        match text_change {
            DiffComponentChange::Modified { entity: e, type_name: tn, diff_string } => {
                assert_eq!(e, entity);
                assert_eq!(tn, type_name);
                assert!(diff_string.contains("Binary(4 bytes)"));
            }
            _ => panic!("Expected Modified variant"),
        }
    }

    #[test] 
    fn test_string_binary_diff() {
        let s1 = "hello".to_string();
        let s2 = "world".to_string();
        
        if let Some(diff) = s1.diff(&s2) {
            match String::diff_to_binary(&diff) {
                Ok(binary_data) => {
                    assert!(!binary_data.is_empty());
                    
                    match String::diff_from_binary(&binary_data) {
                        Ok(restored_diff) => {
                            assert_eq!(diff, restored_diff);
                        }
                        Err(e) => panic!("Failed to deserialize String diff: {}", e),
                    }
                }
                Err(e) => panic!("Failed to serialize String diff: {}", e),
            }
        }
    }

    #[test]
    fn test_binary_diff_change_types() {
        use rust_ecs::ecs::core::Entity;
        
        let entity = Entity::new(1, 0);
        let type_name = "TestType".to_string();
        let diff_data = vec![0x01, 0x02];
        
        // Test Added type
        let added = BinaryDiffComponentChange::from_diff_change_raw(
            entity, type_name.clone(), diff_data.clone(), DiffChangeType::Added
        );
        match added {
            BinaryDiffComponentChange::Added { .. } => {}
            _ => panic!("Expected Added variant"),
        }
        
        // Test Removed type
        let removed = BinaryDiffComponentChange::from_diff_change_raw(
            entity, type_name.clone(), vec![], DiffChangeType::Removed
        );
        match removed {
            BinaryDiffComponentChange::Removed { .. } => {}
            _ => panic!("Expected Removed variant"),
        }
    }
}