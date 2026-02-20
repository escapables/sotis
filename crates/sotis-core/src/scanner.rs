use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use crate::config::FolderEntry;

/// Result of scanning configured folders.
#[derive(Debug)]
pub struct ScanResult {
    pub files: Vec<PathBuf>,
    pub errors: Vec<(PathBuf, String)>,
}

/// Scan configured folders for indexable files.
pub fn scan(folders: &[FolderEntry]) -> ScanResult {
    let mut files = Vec::new();
    let mut errors = Vec::new();

    for folder in folders {
        let extension_filter = normalized_extensions(&folder.extensions);
        let walker = walkdir::WalkDir::new(&folder.path)
            .max_depth(if folder.recursive { usize::MAX } else { 1 })
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| should_traverse(entry.path()));

        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if !entry.file_type().is_file() || is_hidden(path) {
                        continue;
                    }

                    if extension_filter.is_empty() || has_allowed_extension(path, &extension_filter)
                    {
                        files.push(path.to_path_buf());
                    }
                }
                Err(error) => {
                    let path = error
                        .path()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| folder.path.clone());
                    errors.push((path, error.to_string()));
                }
            }
        }
    }

    ScanResult { files, errors }
}

fn normalized_extensions(raw_extensions: &[String]) -> HashSet<String> {
    raw_extensions
        .iter()
        .filter_map(|ext| {
            let trimmed = ext.trim().trim_start_matches('.');
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_ascii_lowercase())
            }
        })
        .collect()
}

fn has_allowed_extension(path: &Path, allowed: &HashSet<String>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|ext| allowed.contains(&ext))
}

fn should_traverse(path: &Path) -> bool {
    !is_hidden(path) && !matches_common_ignored_path(path)
}

fn matches_common_ignored_path(path: &Path) -> bool {
    const IGNORED_NAMES: &[&str] = &[".git", "node_modules", "target", ".cache", "__pycache__"];
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| IGNORED_NAMES.contains(&name))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn non_recursive_scan_only_includes_top_level_files() {
        let base = unique_temp_dir();
        let nested = base.join("nested");
        fs::create_dir_all(&nested).expect("create nested dir");
        fs::write(base.join("root.txt"), "root").expect("write root file");
        fs::write(nested.join("deep.txt"), "deep").expect("write nested file");

        let folders = vec![FolderEntry {
            path: base.clone(),
            recursive: false,
            extensions: vec![],
        }];
        let result = scan(&folders);

        assert!(result.errors.is_empty());
        assert_eq!(result.files.len(), 1);
        assert!(result.files.iter().any(|p| p.ends_with("root.txt")));
        assert!(!result.files.iter().any(|p| p.ends_with("deep.txt")));

        cleanup_temp_dir(&base);
    }

    #[test]
    fn recursive_scan_and_extension_filters_work() {
        let base = unique_temp_dir();
        let nested = base.join("nested");
        fs::create_dir_all(&nested).expect("create nested dir");
        fs::write(base.join("main.rs"), "fn main() {}").expect("write rs file");
        fs::write(nested.join("readme.md"), "# hello").expect("write md file");
        fs::write(nested.join("notes.txt"), "notes").expect("write txt file");

        let folders = vec![FolderEntry {
            path: base.clone(),
            recursive: true,
            extensions: vec![".rs".to_string(), "md".to_string()],
        }];
        let result = scan(&folders);

        assert!(result.errors.is_empty());
        assert_eq!(result.files.len(), 2);
        assert!(result.files.iter().any(|p| p.ends_with("main.rs")));
        assert!(result.files.iter().any(|p| p.ends_with("readme.md")));
        assert!(!result.files.iter().any(|p| p.ends_with("notes.txt")));

        cleanup_temp_dir(&base);
    }

    #[test]
    fn hidden_and_common_ignored_paths_are_skipped() {
        let base = unique_temp_dir();
        let hidden_dir = base.join(".hidden");
        let hidden_file = base.join(".env");
        let git_dir = base.join(".git");
        let node_modules = base.join("node_modules");
        fs::create_dir_all(&hidden_dir).expect("create hidden dir");
        fs::create_dir_all(&git_dir).expect("create git dir");
        fs::create_dir_all(&node_modules).expect("create node_modules dir");
        fs::write(base.join("visible.txt"), "ok").expect("write visible file");
        fs::write(hidden_file, "secret").expect("write hidden file");
        fs::write(hidden_dir.join("secret.txt"), "hidden").expect("write hidden nested");
        fs::write(git_dir.join("config"), "git").expect("write git file");
        fs::write(node_modules.join("pkg.js"), "pkg").expect("write node module file");

        let folders = vec![FolderEntry {
            path: base.clone(),
            recursive: true,
            extensions: vec![],
        }];
        let result = scan(&folders);

        assert!(result.errors.is_empty());
        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].ends_with("visible.txt"));

        cleanup_temp_dir(&base);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sotis-scan-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
