use std::io::Write;
use std::path::Path;

use anyhow::Result;
use tokio::fs;

use crate::extension::ExtensionManifest;
use crate::extensions::ExtensionsToml;
use crate::github;
use crate::survey::Survey;

pub struct ExtensionJsonUsage;

impl Survey for ExtensionJsonUsage {
    async fn run(
        &self,
        work_dir: impl AsRef<Path>,
        extensions_toml: &ExtensionsToml,
    ) -> Result<()> {
        let work_dir = work_dir.as_ref();
        let mut report = Vec::new();

        writeln!(report, "## Extensions using `extension.json`")?;

        for (extension_id, extension) in &extensions_toml.extensions {
            let extension_json_path = extension.extension_dir(work_dir).join("extension.json");
            if !extension_json_path.exists() {
                continue;
            }

            let extension_manifest: ExtensionManifest =
                serde_json_lenient::from_str(&fs::read_to_string(&extension_json_path).await?)?;

            writeln!(report, "- [ ] `{extension_id}`")?;
            if let Some(repository) = extension_manifest.repository.as_ref() {
                writeln!(report, "  - Repository URL: [{repository}]({repository})")?;
            } else {
                writeln!(report, "???")?;
            }

            writeln!(report, "  - Issue: ")?;

            if let Some(repository) = extension_manifest.repository.as_ref() {
                const ZED_DOCS_URL: &str = "https://zed.dev/docs/extensions/developing-extensions#directory-structure-of-a-zed-extension";

                let title = "Migrate to `extension.toml`";
                let mut body = String::new();
                body.push_str("This extension has been identified as still using the legacy `extension.json` manifest format.\n\n");
                body.push_str(&format!("Extensions should use the new `extension.toml` manifest format. See the [Zed extension documentation]({ZED_DOCS_URL}) for more information."));

                let github_issue_url = github::create_github_issue_url(repository, title, &body)?;
                writeln!(report, "    - [Create Issue]({github_issue_url})")?;
            }
        }

        println!("{}", String::from_utf8_lossy(&report));

        Ok(())
    }
}
