use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;

const DOCS_DIR: &str = "docs";
const LEARN_CATALOG_LIMIT: usize = 128;
const MARKDOWN_EXTENSION: &str = "md";
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnCatalogEntry {
    pub title: String,
    pub category: String,
    pub summary: String,
    pub path: PathBuf,
}

pub fn discover_learn_catalog<I>(repo_roots: I) -> Result<Vec<LearnCatalogEntry>, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    let mut entries = Vec::new();
    let mut visited_roots = HashSet::new();
    for repo_root in repo_roots {
        let docs_root = repo_root.join(DOCS_DIR);
        if !docs_root.is_dir() {
            continue;
        }
        let key = normalized_path_key(&docs_root);
        if !visited_roots.insert(key) {
            continue;
        }
        collect_docs(&docs_root, &docs_root, &mut entries)?;
    }
    entries.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.path.cmp(&right.path))
    });
    entries.truncate(LEARN_CATALOG_LIMIT);
    Ok(entries)
}

fn collect_docs(
    docs_root: &Path,
    directory: &Path,
    entries: &mut Vec<LearnCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            collect_docs(docs_root, &path, entries)?;
        } else if file_type.is_file() && is_markdown_file(&path) {
            entries.push(read_learn_doc(docs_root, &path)?);
        }
    }
    Ok(())
}

fn read_learn_doc(docs_root: &Path, path: &Path) -> Result<LearnCatalogEntry, HubError> {
    let text = fs::read_to_string(path)?;
    let title = first_heading(&text).unwrap_or_else(|| fallback_title(path));
    let summary = first_summary_line(&text).unwrap_or_default();
    Ok(LearnCatalogEntry {
        title,
        category: category_from_path(docs_root, path),
        summary,
        path: path.to_path_buf(),
    })
}

fn first_heading(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn first_summary_line(text: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let mut seen_frontmatter_start = false;
    for line in text.lines().map(str::trim) {
        if !seen_frontmatter_start && line == "---" {
            in_frontmatter = true;
            seen_frontmatter_start = true;
            continue;
        }
        if in_frontmatter {
            if line == "---" {
                in_frontmatter = false;
            }
            continue;
        }
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("- ")
            || line.starts_with("```")
            || line.ends_with(':')
        {
            continue;
        }
        return Some(line.to_string());
    }
    None
}

fn category_from_path(docs_root: &Path, path: &Path) -> String {
    path.strip_prefix(docs_root)
        .ok()
        .and_then(|relative| relative.components().next())
        .and_then(|component| component.as_os_str().to_str())
        .map(format_category)
        .unwrap_or_else(|| "Documentation".to_string())
}

fn format_category(value: &str) -> String {
    let mut words = value.replace(['_', '-'], " ");
    if words.trim().is_empty() {
        return "Documentation".to_string();
    }
    let mut chars = words.chars();
    if let Some(first) = chars.next() {
        words = first.to_uppercase().collect::<String>() + chars.as_str();
    }
    words
}

fn fallback_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(format_category)
        .unwrap_or_else(|| "Documentation".to_string())
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(MARKDOWN_EXTENSION))
}

fn should_skip_directory(name: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|skipped| skipped.eq_ignore_ascii_case(name))
}

fn normalized_path_key(path: &Path) -> String {
    let value = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    if cfg!(target_os = "windows") {
        value.to_ascii_lowercase()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn discover_learn_catalog_reads_markdown_titles_and_summaries() {
        let repo_root = temp_repo_root("learn-catalog");
        let docs_root = repo_root.join(DOCS_DIR).join("zircon_hub");
        fs::create_dir_all(&docs_root).unwrap();
        fs::write(
            docs_root.join("index.md"),
            r#"---
related_code:
  - zircon_hub/src/lib.rs
---

# Zircon Hub

`zircon_hub` is the standalone desktop launcher.
"#,
        )
        .unwrap();

        let entries = discover_learn_catalog([repo_root.clone()]).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Zircon Hub");
        assert_eq!(entries[0].category, "Zircon hub");
        assert_eq!(
            entries[0].summary,
            "`zircon_hub` is the standalone desktop launcher."
        );
    }

    #[test]
    fn discover_learn_catalog_skips_non_markdown_and_transient_dirs() {
        let repo_root = temp_repo_root("learn-skip");
        fs::create_dir_all(repo_root.join(DOCS_DIR).join("target")).unwrap();
        fs::write(
            repo_root.join(DOCS_DIR).join("target").join("cache.md"),
            "# Cache",
        )
        .unwrap();
        fs::write(repo_root.join(DOCS_DIR).join("notes.txt"), "ignored").unwrap();

        let entries = discover_learn_catalog([repo_root.clone()]).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert!(entries.is_empty());
    }

    fn temp_repo_root(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("zircon-hub-{label}-{now}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
