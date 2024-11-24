use std::collections::HashMap;

use super::Project;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePackagesHandler {
    dependencies: HashMap<String, String>,
    scripts: HashMap<String, String>,
}

pub struct NodeProject {
    deps: Vec<String>,
}

impl NodeProject {
    pub fn new() -> Self {
        Self { deps: Vec::new() }
    }
}

impl Project for NodeProject {
    const DEPS_FILE: &str = "package.json";
    const ALLOWED_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "scss", "sass", "json"];
    const EXCLUDED_PATHS: &[&str] = &["package.json", "node_modules/"];

    fn parse_deps(&mut self, deps_file_content: &str) -> usize {
        let packages_handler: NodePackagesHandler = serde_json::from_str(deps_file_content)
            .unwrap_or_else(|e| panic!("Cannot parse {} file. {e}", NodeProject::DEPS_FILE));
        self.deps = get_deps_names(packages_handler);
        self.deps.len()
    }

    fn deps(&self) -> &Vec<String> {
        &self.deps
    }
}

fn is_used_in_package_scripts(parsed_file: &NodePackagesHandler, name: &str) -> bool {
    for script in parsed_file.scripts.values() {
        if script.contains(name) {
            return true;
        }
    }
    false
}

fn get_deps_names(parsed_file: NodePackagesHandler) -> Vec<String> {
    let mut names: Vec<String> =
        parsed_file
            .dependencies
            .iter()
            .fold(Vec::new(), |mut acc, (name, _version)| {
                if name.starts_with("@types/") || is_used_in_package_scripts(&parsed_file, name) {
                    return acc;
                }
                acc.push(name.into());
                acc
            });
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn get_deps_names_works() {
        let mut packages_handler = NodePackagesHandler {
            dependencies: HashMap::new(),
            scripts: HashMap::new(),
        };
        packages_handler
            .dependencies
            .insert("foo".into(), "0.1.0".into());
        packages_handler
            .dependencies
            .insert("bar".into(), "0.1.0".into());
        packages_handler
            .dependencies
            .insert("@types/foo".into(), "0.1.0".into());

        assert_eq!(get_deps_names(packages_handler), vec!["bar", "foo"]);
    }

    #[test]
    fn parse_deps_works() {
        let mut project = NodeProject::new();

        let file_content = "{
        \"name\": \"foo\",
        \"dependencies\": {
            \"foo\": \"0.1.0\",
            \"bar\": \"0.1.0\",
            \"bazz\": \"0.1.0\"
        },
        \"devDependencies\": {
            \"dev-foo\": \"0.1.0\",
            \"dev-bar\": \"0.1.0\",
            \"dev-bazz\": \"0.1.0\"
        },
        \"scripts\": {
            \"foo\": \"quix\"
        }
        }";

        assert_eq!(project.parse_deps(file_content), 3);
        assert_eq!(project.deps.len(), 3);
        assert_eq!(project.deps, vec!["bar", "bazz", "foo"]);
    }

    #[test]
    fn guess_if_package_scripts_use_deps() {
        let mut packages_handler = NodePackagesHandler {
            dependencies: HashMap::new(),
            scripts: HashMap::new(),
        };
        packages_handler
            .scripts
            .insert("foo".into(), "foo bar baz".into());

        assert_eq!(is_used_in_package_scripts(&packages_handler, "bar"), true);
        assert_eq!(is_used_in_package_scripts(&packages_handler, "qux"), false);
    }
}
