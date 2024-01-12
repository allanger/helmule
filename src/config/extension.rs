use std::{fs, path::Path};

use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Extension {
    name: Option<String>,
    target_dir: String,
    pub(crate) source_dir: String,
}

impl Extension {
    pub(crate) fn apply(&self, chart_local_path: String) -> Result<(), Box<dyn std::error::Error>> {
        let extension_name = match self.name.clone() {
            Some(res) => res,
            None => "Unnamed".to_string(),
        };
        info!("applying extension: '{}'", extension_name);
        let target_dir = format!("{}/{}", chart_local_path, self.target_dir);
        info!("trying to create a dir: {}", target_dir);
        fs::create_dir(target_dir.clone())?;
        info!("copying {} to {}", self.source_dir, target_dir);
        copy_recursively(self.source_dir.clone(), target_dir)?;
        Ok(())
    }
}

/// Copy files from source to destination recursively.
pub fn copy_recursively(
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
