//! 全量发现的目录遍历器；它只决定清单位置与遍历边界，预算、解析和发布交给访客及发现权威。
//! 找到包清单的目录成为包边界，避免继续把包内实现目录当成独立插件搜索树。
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::PLUGIN_MANIFEST_FILE;

pub(super) const MAX_DISCOVERY_DEPTH: usize = 16;

#[derive(Debug)]
pub(super) enum NativePluginManifestCollectionError {
    EnumerateRoot { root: PathBuf, source: io::Error },
    InspectEntry { root: PathBuf, source: io::Error },
}

impl std::fmt::Display for NativePluginManifestCollectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnumerateRoot { root, source } => write!(
                formatter,
                "failed to enumerate native plugin root {}: {source}",
                root.display()
            ),
            Self::InspectEntry { root, source } => write!(
                formatter,
                "failed to inspect native plugin entry under {}: {source}",
                root.display()
            ),
        }
    }
}

impl std::error::Error for NativePluginManifestCollectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EnumerateRoot { source, .. } | Self::InspectEntry { source, .. } => Some(source),
        }
    }
}

/// 每个文件操作边界都可拒绝继续工作；暂存先准入，诊断通过闭包延迟构造。
/// 生产访客将这些回调绑定到当前票据的取消、期限和资源额度。
pub(super) trait NativePluginManifestTraversalVisitor {
    type Error;

    fn checkpoint(&mut self) -> Result<(), Self::Error>;

    fn reserve_scratch(&mut self, total_bytes: u64) -> Result<(), Self::Error>;

    fn manifest(&mut self, manifest_path: PathBuf) -> Result<(), Self::Error>;

    fn diagnostic(
        &mut self,
        build: impl FnOnce() -> NativePluginManifestTraversalDiagnostic,
    ) -> Result<(), Self::Error>;
}

pub(super) enum NativePluginManifestTraversalError<E> {
    Collection(NativePluginManifestCollectionError),
    Visitor(E),
}

pub(super) enum NativePluginManifestTraversalDiagnostic {
    IgnoredSymbolicLink(PathBuf),
    MaximumDepth { child: PathBuf, maximum: usize },
    OutsideCanonicalRoot(PathBuf),
    CanonicalDirectoryCycle(PathBuf),
}

impl NativePluginManifestTraversalDiagnostic {
    pub(super) fn into_message(self) -> String {
        match self {
            Self::IgnoredSymbolicLink(path) => {
                format!(
                    "native plugin discovery ignored symbolic link {}",
                    path.display()
                )
            }
            Self::MaximumDepth { child, maximum } => format!(
                "native plugin discovery maximum depth {maximum} reached at {}",
                child.display()
            ),
            Self::OutsideCanonicalRoot(child) => format!(
                "native plugin discovery ignored directory outside canonical root: {}",
                child.display()
            ),
            Self::CanonicalDirectoryCycle(child) => format!(
                "native plugin discovery ignored canonical directory cycle at {}",
                child.display()
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct NativePluginManifestTraversal {
    pub(super) enumerated_directories: u64,
    pub(super) inspected_entries: u64,
}

/// 同步执行全量遍历，只由已获准的收集任务调用；忽略符号链接、越根规范目录及过深子树。
/// 遍历错误会携带文件系统原因，访客错误则保留预算或取消类别，供权威分别处理。
pub(super) fn traverse_plugin_manifests<V>(
    root: &Path,
    visitor: &mut V,
) -> Result<NativePluginManifestTraversal, NativePluginManifestTraversalError<V::Error>>
where
    V: NativePluginManifestTraversalVisitor,
{
    visitor
        .checkpoint()
        .map_err(NativePluginManifestTraversalError::Visitor)?;
    let canonical_root = fs::canonicalize(root).map_err(|source| {
        NativePluginManifestTraversalError::Collection(
            NativePluginManifestCollectionError::EnumerateRoot {
                root: root.to_path_buf(),
                source,
            },
        )
    })?;
    let mut retained_scratch_bytes = scratch_bytes_for_path(&canonical_root);
    visitor
        .reserve_scratch(retained_scratch_bytes)
        .map_err(NativePluginManifestTraversalError::Visitor)?;
    let mut visited = HashSet::from([canonical_root.clone()]);
    let mut pending = VecDeque::from([(canonical_root.clone(), 0_usize)]);
    let mut traversal = NativePluginManifestTraversal::default();

    while let Some((directory, depth)) = pending.pop_front() {
        visitor
            .checkpoint()
            .map_err(NativePluginManifestTraversalError::Visitor)?;
        let entries = fs::read_dir(&directory).map_err(|source| {
            NativePluginManifestTraversalError::Collection(
                NativePluginManifestCollectionError::EnumerateRoot {
                    root: directory.clone(),
                    source,
                },
            )
        })?;
        traversal.enumerated_directories = traversal.enumerated_directories.saturating_add(1);

        let mut child_directories = Vec::new();
        let mut package_manifest_found = false;
        let mut entries = entries;
        loop {
            visitor
                .checkpoint()
                .map_err(NativePluginManifestTraversalError::Visitor)?;
            let Some(entry) = entries.next() else {
                break;
            };
            traversal.inspected_entries = traversal.inspected_entries.saturating_add(1);
            let entry = entry.map_err(|source| {
                NativePluginManifestTraversalError::Collection(
                    NativePluginManifestCollectionError::InspectEntry {
                        root: directory.clone(),
                        source,
                    },
                )
            })?;
            visitor
                .checkpoint()
                .map_err(NativePluginManifestTraversalError::Visitor)?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|source| {
                NativePluginManifestTraversalError::Collection(
                    NativePluginManifestCollectionError::InspectEntry {
                        root: directory.clone(),
                        source,
                    },
                )
            })?;
            if file_type.is_symlink() {
                visitor
                    .diagnostic(|| {
                        NativePluginManifestTraversalDiagnostic::IgnoredSymbolicLink(path)
                    })
                    .map_err(NativePluginManifestTraversalError::Visitor)?;
                continue;
            }
            if file_type.is_file()
                && path.file_name().and_then(|value| value.to_str()) == Some(PLUGIN_MANIFEST_FILE)
            {
                visitor
                    .manifest(path)
                    .map_err(NativePluginManifestTraversalError::Visitor)?;
                package_manifest_found = true;
            } else if file_type.is_dir() {
                retained_scratch_bytes =
                    retained_scratch_bytes.saturating_add(scratch_bytes_for_path(&path));
                visitor
                    .reserve_scratch(retained_scratch_bytes)
                    .map_err(NativePluginManifestTraversalError::Visitor)?;
                child_directories.push(path);
            }
        }

        // 包边界在枚举本目录后成立；清单自身解析失败也不把包的子目录重新解释为搜索根。
        if package_manifest_found {
            continue;
        }
        for child in child_directories {
            if depth >= MAX_DISCOVERY_DEPTH {
                visitor
                    .diagnostic(|| NativePluginManifestTraversalDiagnostic::MaximumDepth {
                        child,
                        maximum: MAX_DISCOVERY_DEPTH,
                    })
                    .map_err(NativePluginManifestTraversalError::Visitor)?;
                continue;
            }
            visitor
                .checkpoint()
                .map_err(NativePluginManifestTraversalError::Visitor)?;
            let canonical_child = fs::canonicalize(&child).map_err(|source| {
                NativePluginManifestTraversalError::Collection(
                    NativePluginManifestCollectionError::InspectEntry {
                        root: directory.clone(),
                        source,
                    },
                )
            })?;
            if !canonical_child.starts_with(&canonical_root) {
                visitor
                    .diagnostic(|| {
                        NativePluginManifestTraversalDiagnostic::OutsideCanonicalRoot(child)
                    })
                    .map_err(NativePluginManifestTraversalError::Visitor)?;
                continue;
            }
            if visited.contains(&canonical_child) {
                visitor
                    .diagnostic(|| {
                        NativePluginManifestTraversalDiagnostic::CanonicalDirectoryCycle(child)
                    })
                    .map_err(NativePluginManifestTraversalError::Visitor)?;
                continue;
            }
            retained_scratch_bytes =
                retained_scratch_bytes.saturating_add(scratch_bytes_for_path(&canonical_child));
            visitor
                .reserve_scratch(retained_scratch_bytes)
                .map_err(NativePluginManifestTraversalError::Visitor)?;
            visited.insert(canonical_child.clone());
            pending.push_back((canonical_child, depth + 1));
        }
    }

    Ok(traversal)
}

// 估算保留路径和容器节点所需的准入上界；它是遍历预算模型，不代表分配器测得的实际内存。
fn scratch_bytes_for_path(path: &Path) -> u64 {
    path.as_os_str().len() as u64 + 64
}

#[cfg(test)]
#[path = "tests/collect_manifests.rs"]
mod tests;
