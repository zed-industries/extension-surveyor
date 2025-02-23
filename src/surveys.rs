use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;
use tokio::fs;

use crate::extensions::ExtensionsToml;

pub struct ThemesUsingProperty {
    theme_property: String,
}

impl ThemesUsingProperty {
    pub fn new(theme_property: String) -> Self {
        Self { theme_property }
    }

    pub async fn run(
        &self,
        work_dir: impl AsRef<Path>,
        extensions_toml: &ExtensionsToml,
    ) -> Result<()> {
        let work_dir = work_dir.as_ref();

        for (extension_id, extension) in &extensions_toml.extensions {
            let mut extension_dir = PathBuf::from(work_dir);
            extension_dir.push(&extension.submodule);
            extension_dir.extend(extension.path.as_ref());

            let mut themes_dir = extension_dir.clone();
            themes_dir.push("themes");

            if !fs::try_exists(&themes_dir).await? {
                continue;
            }

            let mut themes = Vec::new();

            let mut themes_entries = fs::read_dir(&themes_dir).await?;
            while let Some(entry) = themes_entries.next_entry().await? {
                let theme_path = entry.path();
                if !theme_path
                    .extension()
                    .map_or(false, |extension| extension == "json")
                {
                    continue;
                }

                let theme = match serde_json_lenient::from_reader::<_, ThemeFamily>(
                    std::fs::File::open(&theme_path)?,
                ) {
                    Ok(theme) => theme,
                    Err(err) => {
                        eprintln!(
                            "{extension_id}: Failed to parse theme file at {theme_path:?}: {err}",
                        );
                        continue;
                    }
                };

                themes.extend(theme.themes);
            }

            for theme in themes {
                if theme.style.contains_key(&self.theme_property) {
                    println!(
                        "{extension_id}: Theme {:?} is using style property {:?}",
                        theme.name, self.theme_property
                    );
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct ThemeFamily {
    pub themes: Vec<Theme>,
}

#[derive(Debug, Deserialize)]
struct Theme {
    pub name: String,
    pub style: BTreeMap<String, serde_json::Value>,
}
