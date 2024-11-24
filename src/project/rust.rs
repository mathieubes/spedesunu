use serde::Deserialize;
use toml::Table;

use super::Project;

#[derive(Deserialize)]
pub struct RustPackagesHandler {
    dependencies: Table,
}

pub struct RustProject {
    deps: Vec<String>,
}

impl RustProject {
    pub fn new() -> Self {
        Self { deps: Vec::new() }
    }
}

impl Project for RustProject {
    const DEPS_FILE: &str = "Cargo.toml";
    const ALLOWED_EXTENSIONS: &[&str] = &["rs"];
    const EXCLUDED_PATHS: &[&str] = &["Cargo.toml"];

    fn parse_deps(&mut self, deps_file_content: &str) -> usize {
        let packages_handler: RustPackagesHandler = toml::from_str(deps_file_content)
            .unwrap_or_else(|e| panic!("Cannot parse {} file. {e}", RustProject::DEPS_FILE));
        self.deps = get_deps_names(packages_handler);
        self.deps.len()
    }

    fn deps(&self) -> &Vec<String> {
        &self.deps
    }
}

fn get_deps_names(parsed_file: RustPackagesHandler) -> Vec<String> {
    let mut names = Vec::from_iter(
        parsed_file
            .dependencies
            .iter()
            .map(|(name, _version)| name.clone()),
    );
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_deps_names_works() {
        let mut packages_handler = RustPackagesHandler {
            dependencies: Table::new(),
        };
        packages_handler
            .dependencies
            .insert("foo".into(), "0.1.0".into());
        packages_handler
            .dependencies
            .insert("bar".into(), "0.1.0".into());

        assert_eq!(get_deps_names(packages_handler), Vec::from(["bar", "foo"]));
    }

    #[test]
    fn parse_deps_works() {
        let mut project = RustProject::new();

        let file_content = "[dependencies]
            foo = \"2.1.0\"
            bar = { version = \"1.0.215\", features = [\"derive\"] }";

        assert_eq!(project.parse_deps(file_content), 2);
        assert_eq!(project.deps.len(), 2);
        assert_eq!(project.deps, Vec::from(["bar", "foo"]));
    }
}
