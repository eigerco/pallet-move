//! A set of IO helpers for accessing assests in `tests/assets` directory.

const MOVE_PROJECTS: &str = "tests/assets/move-projects";

/// Reads bytes from a file for the given path.
/// Can panic if the file doesn't exist.
fn read_bytes(file_path: &str) -> Vec<u8> {
    std::fs::read(file_path)
        .unwrap_or_else(|e| panic!("Can't read {file_path}: {e} - make sure you run pallet-move/tests/assets/move-projects/smove-build-all.sh"))
}

/// Reads a precompiled Move module from our assets directory.
pub fn read_module_from_project(project: &str, module_name: &str) -> Vec<u8> {
    let path =
        format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_modules/{module_name}.mv");

    read_bytes(&path)
}

/// Reads a precompiled Move bundle from our assets directory.
pub fn read_bundle_from_project(project: &str, bundle_name: &str) -> Vec<u8> {
    let path = format!("{MOVE_PROJECTS}/{project}/build/{project}/bundles/{bundle_name}.mvb");

    read_bytes(&path)
}

/// Reads a precompiled Move scripts from our assets directory.
pub fn read_script_from_project(project: &str, script_name: &str) -> Vec<u8> {
    let path =
        format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_scripts/{script_name}.mv");

    read_bytes(&path)
}
