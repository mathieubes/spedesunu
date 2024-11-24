use std::collections::HashSet;

use colored::Colorize;
use walkdir::WalkDir;

use crate::file_handler::{read_file_at_path, string_exists_in_multiline_text};

pub mod node_js;
pub mod rust;

pub trait Project {
    const DEPS_FILE: &str;
    const ALLOWED_EXTENSIONS: &[&str];
    const EXCLUDED_PATHS: &[&str];

    fn parse_deps(&mut self, deps_file_content: &str) -> usize;
    fn deps(&self) -> &Vec<String>;
}

pub struct ScanResult {
    pub deps_count: usize,
    pub scanned_file_count: usize,

    pub unused_deps: HashSet<String>,
}

impl ScanResult {
    fn new() -> Self {
        Self {
            deps_count: 0,
            scanned_file_count: 0,

            unused_deps: HashSet::new(),
        }
    }

    fn print_result() { 
        todo!("Print the result here");
    }
}

pub fn scan_project_deps<T: Project>(mut project: T) -> Result<ScanResult, String> {
    let mut scan_result = ScanResult::new();

    let deps_file_content = read_file_at_path(T::DEPS_FILE)?;
    scan_result.deps_count = project.parse_deps(&deps_file_content);
    println!(
        "{} dependencies found in current project.",
        scan_result.deps_count
    );

    let mut used_deps = HashSet::new();
    for entry in WalkDir::new(".") {
        let entry = entry.unwrap();
        if entry.path().is_dir() || !should_scan_file::<T>(entry.path().to_str().unwrap()) {
            continue;
        }
        scan_result.scanned_file_count += 1;

        let file_content = read_file_at_path(entry.path().to_str().unwrap()).unwrap(); // 😢

        let mut used_deps_in_file = HashSet::new();
        for dep_name in project.deps().iter() {
            if string_exists_in_multiline_text(dep_name, &file_content) {
                used_deps_in_file.insert(dep_name);
            }

        }

        print_current_file_result(&entry.path().display().to_string(), &used_deps_in_file, scan_result.deps_count - used_deps.len());

        for dep_name in used_deps_in_file.into_iter() {
            used_deps.insert(dep_name);
        }
    }

    for dep_name in project.deps().iter() {
        if !used_deps.contains(&dep_name) {
            scan_result.unused_deps.insert(dep_name.to_string());
        }
    }

    Ok(scan_result)
}

fn should_scan_file<T: Project>(file_path: &str) -> bool {
    if file_path == "." {
        return true;
    }

    for excluded in T::EXCLUDED_PATHS.iter() {
        if file_path.contains(excluded) {
            return false;
        }
    }

    for ext in T::ALLOWED_EXTENSIONS.iter() {
        if file_path.ends_with(&format!(".{ext}")) {
            return true;
        }
    }

    false
}

fn print_current_file_result(
    file: &str,
    used_deps_in_file: &HashSet<&String>,
    remaining_unused_deps_count: usize,
) {
    let print_str = format!(
        "==============================
> File : {}
> Deps found in this file : {:?}
> Remaining unused deps count : {}
==============================",
        file, used_deps_in_file, remaining_unused_deps_count
    );

    if used_deps_in_file.is_empty() {
        println!("{}", print_str.red());
    } else {
        println!("{}", print_str);
    }
}

#[cfg(test)]
mod tests {
    use super::{node_js::NodeProject, rust::RustProject, should_scan_file};

    #[test]
    fn node_js_should_scan_file() {
        assert_eq!(should_scan_file::<NodeProject>("foo.js"), true);
        assert_eq!(should_scan_file::<NodeProject>("foo.ts"), true);
        assert_eq!(should_scan_file::<NodeProject>("foo.tsx"), true);
        assert_eq!(should_scan_file::<NodeProject>("foo.jsx"), true);
        assert_eq!(should_scan_file::<NodeProject>("foo.json"), true);
        assert_eq!(should_scan_file::<NodeProject>("foo.scss"), true);
        assert_eq!(should_scan_file::<NodeProject>("foo.sass"), true);
        assert_eq!(should_scan_file::<NodeProject>("foo.rs"), false);
        assert_eq!(should_scan_file::<NodeProject>("foo.jssx"), false);
        assert_eq!(should_scan_file::<NodeProject>("package.json"), false);
        assert_eq!(
            should_scan_file::<NodeProject>("foo/node_modules/foo.ts"),
            false
        );
    }

    #[test]
    fn rust_should_scan_file() {
        assert_eq!(should_scan_file::<RustProject>("foo.rs"), true);
        assert_eq!(should_scan_file::<RustProject>("foo.rss"), false);
        assert_eq!(should_scan_file::<RustProject>("foo.js"), false);
        assert_eq!(should_scan_file::<RustProject>("Cargo.toml"), false);
        assert_eq!(should_scan_file::<RustProject>("Cargo.lock"), false);
        assert_eq!(should_scan_file::<RustProject>("foo.toml"), false);
    }
}
