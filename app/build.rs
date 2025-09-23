use std::path::Path;

use anyhow::anyhow;
use walkdir::WalkDir;

const CSS_TEMPLATES: &str = "styles/templates";
const CSS_COMPILED: &str = "styles/compiled";

fn warning(warning: String) {
    for line in warning.lines() {
        println!("cargo::warning={line}");
    }
}

fn error(error: String) {
    for line in error.lines() {
        println!("cargo::error={line}");
    }
}

fn main() {
    if let Err(err) = build_interoper() {
        warning(format!("failed to build interoper: {err:?}"));
    }
}

fn build_interoper() -> anyhow::Result<()> {
    println!("cargo::rerun-if-changed=Interoper.toml");
    println!("cargo::rerun-if-changed=styles/templates");
    let project = match interoper::build() {
        Ok(project) => project,
        Err(e) => {
            println!("cargo::error=failed on interoper: {e:?}");
            return Err(e);
        }
    };

    let compiled_dir = Path::new(CSS_COMPILED);
    if !std::fs::exists(compiled_dir)? {
        let _ = std::fs::create_dir_all(compiled_dir);
    }

    for entry in WalkDir::new(CSS_TEMPLATES) {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();
        let relative_path = path.strip_prefix(CSS_TEMPLATES)?;

        let Ok(template) = std::fs::read_to_string(path) else {
            continue;
        };
        let mut result = template;

        for (dependency, path) in project.dependencies.iter() {
            let Ok(canonicalized_path_string) =
                std::fs::canonicalize(path)?.into_os_string().into_string()
            else {
                return Err(anyhow!(
                    "failed to canonicalize dependency path for {dependency}"
                ));
            };

            result = result.replace(
                format!("{{{{ interoper:{dependency} }}}}").as_str(),
                canonicalized_path_string.as_str(),
            );
        }

        std::fs::write(Path::new(CSS_COMPILED).join(relative_path), result)?;
    }

    Ok(())
}
