use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct ExtensionsToml {
    #[serde(flatten)]
    pub extensions: BTreeMap<String, ExtensionEntry>,
}

impl ExtensionsToml {
    pub async fn load(root_dir: impl AsRef<Path>) -> Result<Self> {
        let extensions_toml_path = PathBuf::from(root_dir.as_ref()).join("extensions.toml");
        let extensions_toml = fs::read_to_string(&extensions_toml_path).await?;

        Ok(toml::from_str(&extensions_toml)?)
    }
}

#[derive(Debug, Deserialize)]
pub struct ExtensionEntry {
    pub submodule: String,
    #[allow(unused)]
    pub version: String,
    #[serde(default)]
    pub path: Option<String>,
}

impl ExtensionEntry {
    /// Returns the path to the extension directory for this [`ExtensionEntry`].
    pub fn extension_dir(&self, root_dir: &Path) -> PathBuf {
        let mut extension_dir = PathBuf::from(root_dir);
        extension_dir.push(&self.submodule);
        extension_dir.extend(self.path.as_ref());

        extension_dir
    }
}
