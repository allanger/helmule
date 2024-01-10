use std::{fs, path::Path};

use log::info;

/// Copy files from source to destination recursively.
pub(crate) fn copy_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            info!("trying to copy {:?}", entry.path());
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
