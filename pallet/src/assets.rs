//! Helper methods to load move-project test assets.

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
extern crate std;

#[cfg(test)]
use alloc::format;
#[cfg(test)]
use sp_std::vec::Vec;

#[cfg(test)]
const MOVE_PROJECTS: &str = "src/assets/move-projects";

/// Reads bytes from a file for the given path.
/// Can panic if the file doesn't exist.
#[cfg(test)]
fn read_bytes(file_path: &str) -> Vec<u8> {
    std::fs::read(file_path)
        .unwrap_or_else(|e| panic!("Can't read {file_path}: {e} - make sure you run pallet-move/tests/assets/move-projects/smove-build-all.sh"))
}

/// Reads a precompiled Move module from our assets directory.
#[cfg(test)]
pub fn read_module_from_project(project: &str, module_name: &str) -> Vec<u8> {
    let path =
        format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_modules/{module_name}.mv");
    read_bytes(&path)
}

/// Reads a precompiled Move bundle from our assets directory.
#[cfg(test)]
pub fn read_bundle_from_project(project: &str, bundle_name: &str) -> Vec<u8> {
    let path = format!("{MOVE_PROJECTS}/{project}/build/{project}/bundles/{bundle_name}.mvb");
    read_bytes(&path)
}

/// Reads a precompiled Move scripts from our assets directory.
#[cfg(test)]
pub fn read_script_from_project(project: &str, script_name: &str) -> Vec<u8> {
    let path =
        format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_scripts/{script_name}.mv");
    read_bytes(&path)
}

#[macro_export]
macro_rules! script_transaction {
    ($bytecode:expr, $type_args:expr $(, $args:expr)*) => {
        {
            let transaction = ScriptTransaction {
                bytecode: $bytecode,
                type_args: $type_args,
                args: vec![$(bcs::to_bytes($args).unwrap()),*],
            };
            bcs::to_bytes(&transaction).unwrap()
        }
    }
}

#[macro_export]
macro_rules! no_type_args {
    () => {
        vec![]
    };
}
