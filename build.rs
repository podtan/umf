// build.rs - Compile-time UDML specification and URP operations validation
// This build script validates umf.udml.yaml and urp_operations.json at compile time

use std::fs;

fn main() {
    // Validate UDML specification
    println!("cargo:rerun-if-changed=umf.udml.yaml");
    
    let yaml_content = fs::read_to_string("umf.udml.yaml")
        .expect("Failed to read umf.udml.yaml");
    
    match udml::prelude::UdmlDocument::from_yaml(&yaml_content) {
        Ok(doc) => {
            println!("cargo:warning=✓ UDML specification validated: component={} layer={:?}", 
                     doc.id, doc.layer);
        }
        Err(e) => {
            panic!("UDML specification validation failed: {:?}", e);
        }
    }
    
    // Validate URP operations JSON
    println!("cargo:rerun-if-changed=urp_operations.json");
    
    let json_content = fs::read_to_string("urp_operations.json")
        .expect("Failed to read urp_operations.json");
    
    match serde_json::from_str::<serde_json::Value>(&json_content) {
        Ok(ops) => {
            let operation_count = ops.get("operations")
                .and_then(|o| o.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            println!("cargo:warning=✓ URP operations validated: {} operations defined", 
                     operation_count);
        }
        Err(e) => {
            panic!("URP operations JSON validation failed: {:?}", e);
        }
    }
}
