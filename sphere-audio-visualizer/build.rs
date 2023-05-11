//! Compiles the rust implementation of the shaders

use std::path::Path;

use spirv_builder::{MetadataPrintout, SpirvBuilder};

const TARGET: &str = "spirv-unknown-spv1.3";

fn main() {
    let cargo_manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shader_dir = cargo_manifest_dir.join("../sphere-audio-visualizer-spirv");

    rerun_if_changed_recursive(&shader_dir);

    let result = SpirvBuilder::new(shader_dir, TARGET)
        .print_metadata(MetadataPrintout::Full)
        .build()
        .unwrap();

    println!("{:#?}", result);
}

/// Marks every file in a directory recursively as cargo:rerun-if-changed.
fn rerun_if_changed_recursive(path: impl AsRef<Path>) {
    let path = path.as_ref();

    if path.is_dir() {
        for child in path.read_dir().unwrap() {
            rerun_if_changed_recursive(child.unwrap().path())
        }
    }

    if path.is_file() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
