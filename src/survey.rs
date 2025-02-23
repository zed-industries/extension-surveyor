use std::path::Path;

use anyhow::Result;

use crate::extensions::ExtensionsToml;

pub trait Survey {
    async fn run(&self, work_dir: impl AsRef<Path>, extensions_toml: &ExtensionsToml)
        -> Result<()>;
}
