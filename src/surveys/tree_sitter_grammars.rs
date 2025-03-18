use std::collections::HashMap;
use std::io::Write;

use tokio::fs;

use crate::extension::ExtensionManifest;
use crate::survey::Survey;

pub struct TreeSitterGrammars;

impl Survey for TreeSitterGrammars {
    async fn run(
        &self,
        work_dir: impl AsRef<std::path::Path>,
        extensions_toml: &crate::extensions::ExtensionsToml,
    ) -> anyhow::Result<()> {
        let work_dir = work_dir.as_ref();
        let mut report = Vec::new();

        let mut extensions_by_grammar: HashMap<String, Vec<String>> = HashMap::new();

        for (extension_id, extension) in &extensions_toml.extensions {
            let extension_toml_path = extension.extension_dir(work_dir).join("extension.toml");
            if !extension_toml_path.exists() {
                continue;
            }

            let extension_manifest: ExtensionManifest =
                toml::from_str(&fs::read_to_string(&extension_toml_path).await?)?;

            if !extension_manifest.grammars.is_empty() {
                writeln!(report, "- {extension_id}")?;
            }

            for (grammar_name, grammar) in extension_manifest.grammars {
                writeln!(report, "  - {grammar_name}")?;
                writeln!(report, "    - Repo: {}", grammar.repository)?;
                writeln!(report, "    - Rev: {}", grammar.rev)?;
                if let Some(path) = grammar.path.as_ref() {
                    writeln!(report, "    - Path: {}", path)?;
                }

                let full_grammar_path = format!(
                    "{}{}",
                    grammar.repository.trim_end_matches(".git"),
                    grammar
                        .path
                        .map(|path| format!("/{path}"))
                        .unwrap_or_default()
                );
                extensions_by_grammar
                    .entry(full_grammar_path)
                    .or_default()
                    .push(extension_id.clone());
            }
        }

        println!("{}", String::from_utf8_lossy(&report));

        let dupes = extensions_by_grammar
            .iter()
            .filter(|(_, extensions)| extensions.len() > 1)
            .collect::<Vec<_>>();

        if !dupes.is_empty() {
            println!("Grammars provided by multiple extensions:");
            for (grammar, extensions) in dupes {
                println!("Grammar: {}", grammar);
                for extension in extensions {
                    println!("  - {}", extension);
                }
            }
        }

        Ok(())
    }
}
