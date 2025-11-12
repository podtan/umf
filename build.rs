// build.rs - Compile-time UDML specification validation
// This build script validates umf.udml.yaml at compile time using the udml crate

use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=umf.udml.yaml");
    
    // Read and validate the UDML YAML file at compile time
    let yaml_content = fs::read_to_string("umf.udml.yaml")
        .expect("Failed to read umf.udml.yaml");
    
    // Parse using udml crate to validate at compile time
    match udml::prelude::UdmlDocument::from_yaml(&yaml_content) {
        Ok(doc) => {
            println!("cargo:warning=✓ UDML specification validated: component={} layer={:?}", 
                     doc.id, doc.layer);
        }
        Err(e) => {
            panic!("UDML specification validation failed: {:?}", e);
        }
    }
}
