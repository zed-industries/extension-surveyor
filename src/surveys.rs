use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use tokio::fs;
use url::Url;

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
        let mut report = Vec::new();

        for (extension_id, extension) in &extensions_toml.extensions {
            let mut themes_dir = extension.extension_dir(work_dir);
            themes_dir.push("themes");

            if !fs::try_exists(&themes_dir).await? {
                continue;
            }

            let extension_manifest: ExtensionManifest = {
                let extension_manifest_path =
                    extension.extension_dir(work_dir).join("extension.toml");
                if extension_manifest_path.exists() {
                    toml::from_str(&fs::read_to_string(&extension_manifest_path).await?)?
                } else {
                    let extension_manifest_json_path =
                        extension.extension_dir(work_dir).join("extension.json");
                    serde_json_lenient::from_str(
                        &fs::read_to_string(&extension_manifest_json_path).await?,
                    )?
                }
            };

            fn write_extension_header(
                report: &mut Vec<u8>,
                extension_id: &str,
                extension_manifest: &ExtensionManifest,
            ) -> Result<()> {
                writeln!(report, "- [ ] `{extension_id}`")?;
                write!(report, "  - Repository: ")?;
                if let Some(repository) = extension_manifest.repository.as_ref() {
                    writeln!(report, "[{repository}]({repository})")?;
                } else {
                    writeln!(report, "???")?;
                }

                writeln!(report, "  - Issue: TBD")?;

                Ok(())
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
                        write_extension_header(&mut report, extension_id, &extension_manifest)?;
                        writeln!(report, "  - Errors:")?;
                        writeln!(
                            &mut report,
                            "    - Failed to parse theme file at {theme_path:?}: {err}"
                        )?;

                        continue;
                    }
                };

                themes.extend(theme.themes);
            }

            let themes_using_property = themes
                .into_iter()
                .filter(|theme| theme.style.contains_key(&self.theme_property))
                .collect::<Vec<_>>();
            if themes_using_property.is_empty() {
                continue;
            }

            write_extension_header(&mut report, extension_id, &extension_manifest)?;

            if let Some(repository) = &extension_manifest.repository {
                let mut github_issue_url = Url::parse(repository)?;
                github_issue_url
                    .path_segments_mut()
                    .map_err(|_| anyhow!("invalid repository URL"))?
                    .extend(["issues", "new"]);
                let mut query = github_issue_url.query_pairs_mut();
                query.append_pair(
                    "title",
                    &format!("Deprecated `{}` usage", self.theme_property),
                );

                let mut issue_body = String::new();
                issue_body.push_str("This extension has been identified as using the deprecated `scrollbar_thumb.background` style property.\n\n");
                issue_body.push_str("This property has been deprecated in favor of `scrollbar.thumb.background`. Please migrate to using the new property.\n\n");
                issue_body.push_str("The following themes are impacted:\n\n");

                for theme in &themes_using_property {
                    issue_body.push_str(&format!(
                        "- Theme {:?} is using deprecated style property `{}`\n",
                        theme.name, self.theme_property
                    ));
                }

                query.append_pair("body", &issue_body);

                query.finish();
                drop(query);

                writeln!(&mut report, "    - [Create Issue]({github_issue_url})")?;
            }

            writeln!(&mut report, "  - Errors:")?;

            for theme in &themes_using_property {
                writeln!(
                    &mut report,
                    "    - Theme {:?} is using deprecated style property `{}`",
                    theme.name, self.theme_property
                )?;
            }
        }

        println!("{}", String::from_utf8_lossy(&report));

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct ExtensionManifest {
    repository: Option<String>,
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
