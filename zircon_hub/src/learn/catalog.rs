use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::projects::project_filesystem_path_key;

const DOCS_DIR: &str = "docs";
const LEARN_CATALOG_LIMIT: usize = 128;
const MARKDOWN_EXTENSION: &str = "md";
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];
pub const SELECTED_PROJECT_LEARN_SOURCE: &str = "Selected Project";
pub const SOURCE_ENGINE_LEARN_SOURCE: &str = "Source Engine";

/// 学习页展示的文档摘要；文件路径仍是打开文档动作的真实目标。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnCatalogEntry {
    pub title: String,
    pub category: String,
    pub source: String,
    pub summary: String,
    pub path: PathBuf,
}

#[derive(Clone)]
struct RankedLearnCatalogEntry {
    root_rank: usize,
    entry: LearnCatalogEntry,
}

pub fn discover_learn_catalog<I>(repo_roots: I) -> Result<Vec<LearnCatalogEntry>, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    discover_learn_catalog_for_scope(None, repo_roots)
}

/// 以当前项目和源码引擎候选根重建学习目录；调用方在作用域变化后刷新结果。
/// 来源优先级和数量上限属于展示契约，不能据此判定某个磁盘文档不存在。
pub fn discover_learn_catalog_for_scope<I>(
    selected_project_root: Option<PathBuf>,
    repo_roots: I,
) -> Result<Vec<LearnCatalogEntry>, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    let mut entries = Vec::new();
    let mut visited_roots = HashSet::new();

    if let Some(project_root) = selected_project_root {
        collect_docs_root(
            SELECTED_PROJECT_LEARN_SOURCE,
            &project_root,
            0,
            &mut visited_roots,
            &mut entries,
        )?;
    }

    for (root_rank, repo_root) in repo_roots.into_iter().enumerate() {
        collect_docs_root(
            SOURCE_ENGINE_LEARN_SOURCE,
            &repo_root,
            root_rank,
            &mut visited_roots,
            &mut entries,
        )?;
    }

    retain_top_ranked_entries(&mut entries);
    Ok(entries.into_iter().map(|ranked| ranked.entry).collect())
}

fn collect_docs_root(
    source: &str,
    repo_root: &Path,
    root_rank: usize,
    visited_roots: &mut HashSet<String>,
    entries: &mut Vec<RankedLearnCatalogEntry>,
) -> Result<(), HubError> {
    let docs_root = repo_root.join(DOCS_DIR);
    if !docs_root.is_dir() {
        return Ok(());
    }
    let key = project_filesystem_path_key(&docs_root);
    if !visited_roots.insert(key) {
        return Ok(());
    }
    collect_docs(source, &docs_root, &docs_root, root_rank, entries)
}

fn source_priority(source: &str) -> u8 {
    match source {
        SELECTED_PROJECT_LEARN_SOURCE => 0,
        SOURCE_ENGINE_LEARN_SOURCE => 1,
        _ => 2,
    }
}

fn ranked_learn_order(left: &RankedLearnCatalogEntry, right: &RankedLearnCatalogEntry) -> Ordering {
    source_priority(&left.entry.source)
        .cmp(&source_priority(&right.entry.source))
        .then_with(|| left.root_rank.cmp(&right.root_rank))
        .then_with(|| left.entry.source.cmp(&right.entry.source))
        .then_with(|| left.entry.category.cmp(&right.entry.category))
        .then_with(|| left.entry.title.cmp(&right.entry.title))
        .then_with(|| left.entry.path.cmp(&right.entry.path))
}

// 在限制列表规模前先保护当前项目与首选源码根的优先级。
fn retain_top_ranked_entries(entries: &mut Vec<RankedLearnCatalogEntry>) {
    if entries.len() > LEARN_CATALOG_LIMIT {
        entries.select_nth_unstable_by(LEARN_CATALOG_LIMIT, ranked_learn_order);
        entries.truncate(LEARN_CATALOG_LIMIT);
    }
    entries.sort_by(ranked_learn_order);
}

fn collect_docs(
    source: &str,
    docs_root: &Path,
    directory: &Path,
    root_rank: usize,
    entries: &mut Vec<RankedLearnCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            collect_docs(source, docs_root, &path, root_rank, entries)?;
        } else if file_type.is_file() && is_markdown_file(&path) {
            entries.push(RankedLearnCatalogEntry {
                root_rank,
                entry: read_learn_doc(source, docs_root, &path)?,
            });
        }
    }
    Ok(())
}

fn read_learn_doc(
    source: &str,
    docs_root: &Path,
    path: &Path,
) -> Result<LearnCatalogEntry, HubError> {
    let text = fs::read_to_string(path)?;
    let title = first_heading(&text).unwrap_or_else(|| fallback_title(path));
    let summary = first_summary_line(&text).unwrap_or_default();
    Ok(LearnCatalogEntry {
        title,
        category: category_from_path(docs_root, path),
        source: source.to_string(),
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

#[cfg(test)]
#[path = "tests/catalog.rs"]
mod tests;
