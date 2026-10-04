//! 被选中包的字节码按规范路径单飞读取并受缓存预算约束；缓存命中依据文件元数据指纹，装载前仍重查路径包含关系。
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::SystemTime;

use crate::script::{VmError, VmPluginPackage};

use super::{DiscoveredVmPluginPackage, VmPluginDiscoveryLimits};

#[derive(Debug)]
pub(crate) struct VmPluginPayloadCache {
    limits: VmPluginDiscoveryLimits,
    state: Mutex<PayloadCacheState>,
}

#[derive(Debug, Default)]
struct PayloadCacheState {
    entries: HashMap<PathBuf, Arc<CachedPayload>>,
    retained_bytes: usize,
}

#[derive(Debug)]
struct CachedPayload {
    fingerprint: PayloadFingerprint,
    bytes: OnceLock<Result<Arc<[u8]>, Arc<str>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PayloadFingerprint {
    len: u64,
    modified: Option<SystemTime>,
}

impl Default for VmPluginPayloadCache {
    fn default() -> Self {
        Self::new(VmPluginDiscoveryLimits::default())
    }
}

impl VmPluginPayloadCache {
    pub(crate) fn new(limits: VmPluginDiscoveryLimits) -> Self {
        Self {
            limits,
            state: Mutex::new(PayloadCacheState::default()),
        }
    }

    pub(crate) fn materialize(
        &self,
        discovered: &DiscoveredVmPluginPackage,
    ) -> Result<VmPluginPackage, VmError> {
        if !discovered.package.bytecode.is_empty() || discovered.package.zr_vm_project.is_some() {
            return Ok(discovered.package.clone());
        }
        let Some(bytecode_path) = discovered.source.bytecode_path.as_deref() else {
            return Ok(discovered.package.clone());
        };
        let package_root = discovered.source.package_root.as_deref().ok_or_else(|| {
            VmError::Operation(format!(
                "discovered bytecode {} has no package root",
                bytecode_path.display()
            ))
        })?;
        let canonical_path = contained_regular_file(package_root, bytecode_path, "bytecode")?;
        let bytes = self.load_canonical_path(&canonical_path)?;
        let mut package = discovered.package.clone();
        package.bytecode = bytes.as_ref().to_vec();
        Ok(package)
    }

    #[cfg(test)]
    pub(super) fn load_path(&self, path: &Path) -> Result<Arc<[u8]>, VmError> {
        let parent = path.parent().ok_or_else(|| {
            VmError::Operation(format!("bytecode path has no parent: {}", path.display()))
        })?;
        let canonical_path = contained_regular_file(parent, path, "bytecode")?;
        self.load_canonical_path(&canonical_path)
    }

    fn load_canonical_path(&self, path: &Path) -> Result<Arc<[u8]>, VmError> {
        let metadata = fs::metadata(path).map_err(|error| {
            VmError::Operation(format!(
                "failed to inspect plugin bytecode {}: {error}",
                path.display()
            ))
        })?;
        if metadata.len() > self.limits.max_bytecode_bytes as u64 {
            return Err(VmError::Operation(format!(
                "plugin bytecode {} exceeds bytecode byte budget {}",
                path.display(),
                self.limits.max_bytecode_bytes
            )));
        }
        // TODO: [CR-SCRIPT-AUDIT-0003] 确认发布流程保证字节码变化必改变长度或修改时间；同长度且保留时间戳的替换会复用旧载荷，缺少该场景的缓存失效测试。
        let fingerprint = PayloadFingerprint {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        };
        let payload_bytes = usize::try_from(fingerprint.len).map_err(|_| {
            VmError::Operation(format!(
                "plugin bytecode size cannot fit host usize: {}",
                path.display()
            ))
        })?;
        let entry = {
            let mut state = self.state_lock();
            if let Some(current) = state.entries.get(path) {
                if current.fingerprint == fingerprint {
                    Arc::clone(current)
                } else {
                    let current_bytes = usize::try_from(current.fingerprint.len).map_err(|_| {
                        VmError::Operation(format!(
                            "cached plugin bytecode size cannot fit host usize: {}",
                            path.display()
                        ))
                    })?;
                    let retained_without_current =
                        state.retained_bytes.saturating_sub(current_bytes);
                    let next_retained = retained_without_current
                        .checked_add(payload_bytes)
                        .ok_or_else(|| {
                            VmError::Operation(
                                "plugin bytecode cache byte counter overflowed".to_string(),
                            )
                        })?;
                    self.check_retained_bytes(next_retained)?;
                    let replacement = Arc::new(CachedPayload {
                        fingerprint,
                        bytes: OnceLock::new(),
                    });
                    state
                        .entries
                        .insert(path.to_path_buf(), Arc::clone(&replacement));
                    state.retained_bytes = next_retained;
                    replacement
                }
            } else {
                if state.entries.len() >= self.limits.max_cached_bytecode_entries {
                    return Err(VmError::Operation(format!(
                        "plugin bytecode cache entry budget {} is exhausted",
                        self.limits.max_cached_bytecode_entries
                    )));
                }
                let next_retained =
                    state
                        .retained_bytes
                        .checked_add(payload_bytes)
                        .ok_or_else(|| {
                            VmError::Operation(
                                "plugin bytecode cache byte counter overflowed".to_string(),
                            )
                        })?;
                self.check_retained_bytes(next_retained)?;
                let inserted = Arc::new(CachedPayload {
                    fingerprint,
                    bytes: OnceLock::new(),
                });
                state
                    .entries
                    .insert(path.to_path_buf(), Arc::clone(&inserted));
                state.retained_bytes = next_retained;
                inserted
            }
        };
        match entry.bytes.get_or_init(|| {
            read_bounded_file_with_expected_bytes(
                path,
                self.limits.max_bytecode_bytes,
                "plugin bytecode",
                Some(payload_bytes),
            )
            .map(Arc::<[u8]>::from)
            .map_err(|error| Arc::<str>::from(error.to_string()))
        }) {
            Ok(bytes) => Ok(Arc::clone(bytes)),
            Err(error) => {
                let mut state = self.state_lock();
                let remove_failed = state
                    .entries
                    .get(path)
                    .is_some_and(|current| Arc::ptr_eq(current, &entry));
                if remove_failed {
                    state.entries.remove(path);
                    state.retained_bytes = state.retained_bytes.saturating_sub(payload_bytes);
                }
                Err(VmError::Operation(error.to_string()))
            }
        }
    }

    fn check_retained_bytes(&self, retained_bytes: usize) -> Result<(), VmError> {
        if retained_bytes > self.limits.max_cached_bytecode_bytes {
            return Err(VmError::Operation(format!(
                "plugin bytecode cache byte budget {} is exhausted",
                self.limits.max_cached_bytecode_bytes
            )));
        }
        Ok(())
    }

    fn state_lock(&self) -> MutexGuard<'_, PayloadCacheState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(super) fn read_bounded_file(
    path: &Path,
    max_bytes: usize,
    description: &str,
) -> Result<Vec<u8>, VmError> {
    read_bounded_file_with_expected_bytes(path, max_bytes, description, None)
}

fn read_bounded_file_with_expected_bytes(
    path: &Path,
    max_bytes: usize,
    description: &str,
    expected_bytes: Option<usize>,
) -> Result<Vec<u8>, VmError> {
    let file = File::open(path).map_err(|error| {
        VmError::Operation(format!(
            "failed to read {description} {}: {error}",
            path.display()
        ))
    })?;
    let read_limit = max_bytes.saturating_add(1);
    let expected_bytes = expected_bytes
        .or_else(|| {
            file.metadata()
                .ok()
                .and_then(|metadata| usize::try_from(metadata.len()).ok())
        })
        .unwrap_or(0)
        .min(read_limit);
    let bytes = read_bounded_stream(file, expected_bytes, max_bytes).map_err(|error| {
        VmError::Operation(format!(
            "failed to read {description} {}: {error}",
            path.display()
        ))
    })?;
    if bytes.len() > max_bytes {
        return Err(VmError::Operation(format!(
            "{description} {} exceeds byte budget {max_bytes}",
            path.display()
        )));
    }
    Ok(bytes)
}

fn read_bounded_stream(
    reader: impl Read,
    expected_bytes: usize,
    max_bytes: usize,
) -> io::Result<Vec<u8>> {
    let read_limit = max_bytes.saturating_add(1);
    let mut bytes = Vec::with_capacity(expected_bytes.min(read_limit));
    reader
        .take(u64::try_from(read_limit).unwrap_or(u64::MAX))
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn contained_regular_file(
    package_root: &Path,
    path: &Path,
    description: &str,
) -> Result<PathBuf, VmError> {
    let link_metadata = fs::symlink_metadata(path).map_err(|error| {
        VmError::Operation(format!(
            "failed to inspect plugin {description} {}: {error}",
            path.display()
        ))
    })?;
    if link_metadata.file_type().is_symlink() {
        return Err(VmError::Operation(format!(
            "plugin {description} cannot be a symbolic link: {}",
            path.display()
        )));
    }
    if !link_metadata.is_file() {
        return Err(VmError::Operation(format!(
            "plugin {description} is not a regular file: {}",
            path.display()
        )));
    }
    let canonical_root = package_root.canonicalize().map_err(|error| {
        VmError::Operation(format!(
            "failed to resolve plugin package root {}: {error}",
            package_root.display()
        ))
    })?;
    let canonical_path = path.canonicalize().map_err(|error| {
        VmError::Operation(format!(
            "failed to resolve plugin {description} {}: {error}",
            path.display()
        ))
    })?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(VmError::Operation(format!(
            "plugin {description} escapes package root {}: {}",
            canonical_root.display(),
            canonical_path.display()
        )));
    }
    Ok(canonical_path)
}

#[cfg(test)]
#[path = "tests/payload_cache.rs"]
mod tests;
