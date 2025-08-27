use std::io::Write;
use std::path::Path;

use anyhow::Result;
use tokio::fs;
use tokio::io::AsyncReadExt;

use crate::extension::ExtensionManifest;
use crate::extensions::{ExtensionsToml, ThemeFamily};
use crate::github;
use crate::survey::Survey;

pub struct ThemePropertyUsage {
    theme_property: Vec<String>,
}

impl ThemePropertyUsage {
    pub fn new(theme_property: Vec<String>) -> Self {
        Self { theme_property }
    }

    pub fn property_name(&self) -> String {
        self.theme_property
            .iter()
            .map(|property| format!("`{property}`"))
            .rev()
            .collect::<Vec<String>>()
            .join(" in section ")
    }
}

impl Survey for ThemePropertyUsage {
    async fn run(
        &self,
        work_dir: impl AsRef<Path>,
        extensions_toml: &ExtensionsToml,
    ) -> Result<()> {
        let work_dir = work_dir.as_ref();
        let mut report = Vec::new();

        let property_iterator = self.theme_property.iter();

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
                if theme_path
                    .extension()
                    .is_none_or(|extension| extension != "json")
                {
                    continue;
                }

                let mut buf = String::new();
                fs::File::open(&theme_path)
                    .await?
                    .read_to_string(&mut buf)
                    .await?;
                let theme = match serde_json_lenient::from_str_lenient::<ThemeFamily>(&buf) {
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
                .filter(|theme| {
                    property_iterator
                        .clone()
                        .fold(Some(&theme.style), |style, key| {
                            style.and_then(|s| s.as_object().and_then(|obj| obj.get(key)))
                        })
                        .is_some()
                })
                .collect::<Vec<_>>();
            if themes_using_property.is_empty() {
                continue;
            }

            write_extension_header(&mut report, extension_id, &extension_manifest)?;

            if let Some(repository) = &extension_manifest.repository {
                let title = format!("Deprecated `{}` usage", self.property_name());
                let mut issue_body = String::new();
                issue_body.push_str("This extension has been identified as using the deprecated `scrollbar_thumb.background` style property.\n\n");
                issue_body.push_str("This property has been deprecated in favor of `scrollbar.thumb.background`. Please migrate to using the new property.\n\n");
                issue_body.push_str("The following themes are impacted:\n\n");

                for theme in &themes_using_property {
                    issue_body.push_str(&format!(
                        "- Theme {:?} is using deprecated style property {}\n",
                        theme.name,
                        self.property_name()
                    ));
                }

                let github_issue_url =
                    github::create_github_issue_url(repository, &title, &issue_body)?;

                writeln!(&mut report, "    - [Create Issue]({github_issue_url})")?;
            }

            writeln!(&mut report, "  - Errors:")?;

            for theme in &themes_using_property {
                writeln!(
                    &mut report,
                    "    - Theme {:?} is using deprecated style property {}",
                    theme.name,
                    self.property_name()
                )?;
            }
        }

        println!("{}", String::from_utf8_lossy(&report));

        Ok(())
    }
}
