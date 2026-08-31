use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::RelPath;

use crate::asset::project::ProjectPaths;
use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::{AssetImporter, AssetImporterDescriptor};

use super::resolver_index::MigrationSourceProjection;

pub(super) struct RecognizedSource {
    pub(super) path: PathBuf,
    pub(super) descriptor: AssetImporterDescriptor,
    pub(super) root_relative_identities: Vec<RootRelativePhysicalIdentity>,
}

#[derive(Clone)]
pub(super) struct RootRelativePhysicalIdentity {
    pub(super) logical_root: RelPath,
    pub(super) root: PathBuf,
    pub(super) relative: PathBuf,
}

struct MigrationPhysicalIdentity {
    path: PathBuf,
    physical_path: PathBuf,
    root_relative_identities: Vec<RootRelativePhysicalIdentity>,
}

struct MigrationRoot {
    logical_root: RelPath,
    walk_path: PathBuf,
    physical_path: PathBuf,
}

#[derive(Default)]
struct MigrationInventoryStats {
    directory_visits: usize,
    directory_sorts: usize,
    file_visits: usize,
}

pub(super) struct MigrationInventory {
    recognized_sources: Vec<RecognizedSource>,
    authoring_files: Vec<PathBuf>,
    sidecar_candidates: Vec<PathBuf>,
    transaction_targets: Vec<PathBuf>,
    rejected_paths: Vec<PathBuf>,
    physical_identities: Vec<MigrationPhysicalIdentity>,
    stats: MigrationInventoryStats,
}

impl MigrationInventory {
    pub(super) fn build(roots: &[(RelPath, PathBuf)]) -> Result<Self, std::io::Error> {
        let roots = prepare_roots(roots)?;
        let mut builder = MigrationInventoryBuilder::new(roots);
        for root_index in 0..builder.roots.len() {
            let root = builder.roots[root_index].walk_path.clone();
            builder.walk(&root)?;
        }
        Ok(builder.finish())
    }

    pub(super) fn recognized_sources(&self) -> &[RecognizedSource] {
        &self.recognized_sources
    }

    pub(super) fn authoring_files(&self) -> &[PathBuf] {
        &self.authoring_files
    }

    pub(super) fn sidecar_candidates(&self) -> &[PathBuf] {
        &self.sidecar_candidates
    }

    pub(super) fn transaction_targets(&self) -> &[PathBuf] {
        &self.transaction_targets
    }

    pub(super) fn is_rejected_path(&self, path: &Path) -> bool {
        self.rejected_paths
            .binary_search_by(|candidate| candidate.as_path().cmp(path))
            .is_ok()
    }

    pub(super) fn entry_visits(&self) -> usize {
        self.stats.file_visits
    }

    pub(super) fn directory_reads(&self) -> usize {
        self.stats.directory_visits
    }

    pub(super) fn directory_sorts(&self) -> usize {
        self.stats.directory_sorts
    }

    /// Projects the one inventory generation into resolver-owned lookup records.
    ///
    /// The scan owns canonicalization and link rejection. The resulting index is therefore a
    /// pure in-memory view and must never repeat filesystem validation per reference.
    pub(super) fn resolver_projections(&self) -> Vec<MigrationSourceProjection> {
        self.physical_identities
            .iter()
            .flat_map(|physical_identity| {
                let physical_path = physical_identity.physical_path.clone();
                physical_identity
                    .root_relative_identities
                    .iter()
                    .filter_map(move |root_identity| {
                        let relative = root_identity.relative.to_string_lossy().replace('\\', "/");
                        let root_relative = RelPath::parse(relative).ok()?;
                        Some(MigrationSourceProjection::new(
                            root_identity.logical_root.clone(),
                            root_identity.root.clone(),
                            root_relative,
                            physical_path.clone(),
                        ))
                    })
            })
            .collect()
    }

    pub(super) fn physical_path_for(&self, path: &Path) -> Option<&Path> {
        self.physical_identities
            .binary_search_by(|identity| identity.path.as_path().cmp(path))
            .ok()
            .map(|index| &self.physical_identities[index])
            .map(|identity| identity.physical_path.as_path())
    }
}

struct MigrationInventoryBuilder {
    importer: AssetImporter,
    roots: Vec<MigrationRoot>,
    visited_directories: HashSet<PathBuf>,
    recognized_sources: Vec<RecognizedSource>,
    authoring_files: Vec<PathBuf>,
    sidecar_candidates: Vec<PathBuf>,
    transaction_targets: Vec<PathBuf>,
    rejected_paths: Vec<PathBuf>,
    physical_identities: Vec<MigrationPhysicalIdentity>,
    stats: MigrationInventoryStats,
}

impl MigrationInventoryBuilder {
    fn new(roots: Vec<MigrationRoot>) -> Self {
        Self {
            importer: AssetImporter::default(),
            roots,
            visited_directories: HashSet::new(),
            recognized_sources: Vec::new(),
            authoring_files: Vec::new(),
            sidecar_candidates: Vec::new(),
            transaction_targets: Vec::new(),
            rejected_paths: Vec::new(),
            physical_identities: Vec::new(),
            stats: MigrationInventoryStats::default(),
        }
    }

    fn walk(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let metadata = fs::symlink_metadata(path)?;
        if is_link_or_reparse(&metadata) {
            self.rejected_paths.push(path.to_path_buf());
            return Ok(());
        }
        if metadata.is_file() {
            self.record_file(path)?;
            return Ok(());
        }
        if !metadata.is_dir() {
            return Ok(());
        }
        let physical_directory = ProjectPaths::resolve_existing_path(path)?;
        if !self.visited_directories.insert(physical_directory) {
            return Ok(());
        }

        self.stats.directory_visits += 1;
        let mut children = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        children.sort_unstable_by_key(|entry| entry.file_name());
        self.stats.directory_sorts += 1;
        for child in children {
            if child.file_name() != ".zircon" {
                self.walk(&child.path())?;
            }
        }
        Ok(())
    }

    fn record_file(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.stats.file_visits += 1;
        let physical_path = ProjectPaths::resolve_existing_path(path)?;
        let root_relative_identities = self.root_relative_identities(&physical_path);
        self.physical_identities.push(MigrationPhysicalIdentity {
            path: path.to_path_buf(),
            physical_path,
            root_relative_identities: root_relative_identities.clone(),
        });
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        if is_sidecar_name(name) {
            self.transaction_targets.push(path.to_path_buf());
            if let Some(counterpart) = sidecar_counterpart(path) {
                self.transaction_targets.push(counterpart);
            }
            self.sidecar_candidates.push(path.to_path_buf());
            return Ok(());
        }

        if is_supported_authoring_file(path) {
            self.authoring_files.push(path.to_path_buf());
            self.transaction_targets.push(path.to_path_buf());
        }
        if !is_auxiliary_source(path) {
            if let Ok(descriptor) = self.importer.descriptor_for_source(path) {
                self.recognized_sources.push(RecognizedSource {
                    path: path.to_path_buf(),
                    descriptor,
                    root_relative_identities,
                });
            }
        }
        Ok(())
    }

    fn root_relative_identities(&self, physical_path: &Path) -> Vec<RootRelativePhysicalIdentity> {
        let mut identities = self
            .roots
            .iter()
            .filter_map(|root| {
                physical_path
                    .strip_prefix(&root.physical_path)
                    .ok()
                    .map(|relative| RootRelativePhysicalIdentity {
                        logical_root: root.logical_root.clone(),
                        root: root.physical_path.clone(),
                        relative: relative.to_path_buf(),
                    })
            })
            .collect::<Vec<_>>();
        identities.sort_unstable_by(|left, right| {
            left.root
                .cmp(&right.root)
                .then_with(|| left.relative.cmp(&right.relative))
                .then_with(|| left.logical_root.cmp(&right.logical_root))
        });
        identities.dedup_by(|left, right| {
            left.root == right.root
                && left.relative == right.relative
                && left.logical_root == right.logical_root
        });
        identities
    }

    fn finish(mut self) -> MigrationInventory {
        self.recognized_sources
            .sort_by(|left, right| left.path.cmp(&right.path));
        self.recognized_sources
            .dedup_by(|left, right| left.path == right.path);
        for source in &self.recognized_sources {
            if let Some(name) = source.path.file_name().and_then(|name| name.to_str()) {
                self.transaction_targets
                    .push(source.path.with_file_name(format!("{name}.zmeta")));
            }
        }
        sort_dedup(&mut self.authoring_files);
        sort_dedup(&mut self.sidecar_candidates);
        sort_dedup(&mut self.transaction_targets);
        sort_dedup(&mut self.rejected_paths);
        self.physical_identities
            .sort_by(|left, right| left.path.cmp(&right.path));
        self.physical_identities
            .dedup_by(|left, right| left.path == right.path);
        MigrationInventory {
            recognized_sources: self.recognized_sources,
            authoring_files: self.authoring_files,
            sidecar_candidates: self.sidecar_candidates,
            transaction_targets: self.transaction_targets,
            rejected_paths: self.rejected_paths,
            physical_identities: self.physical_identities,
            stats: self.stats,
        }
    }
}

fn prepare_roots(roots: &[(RelPath, PathBuf)]) -> Result<Vec<MigrationRoot>, std::io::Error> {
    let mut roots = roots.to_vec();
    roots.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    roots.dedup();
    let mut prepared = Vec::with_capacity(roots.len());
    for (logical_root, walk_path) in roots {
        let metadata = match fs::symlink_metadata(&walk_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if is_link_or_reparse(&metadata) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "migration asset root '{}' is a symbolic link or reparse point",
                    ProjectPaths::display_path(&walk_path).display()
                ),
            ));
        }
        if !metadata.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "migration asset root '{}' is not a directory",
                    ProjectPaths::display_path(&walk_path).display()
                ),
            ));
        }
        let physical_path = ProjectPaths::resolve_existing_path(&walk_path)?;
        prepared.push(MigrationRoot {
            logical_root,
            walk_path,
            physical_path,
        });
    }
    prepared.sort_by(|left, right| {
        left.physical_path
            .cmp(&right.physical_path)
            .then_with(|| left.logical_root.cmp(&right.logical_root))
            .then_with(|| left.walk_path.cmp(&right.walk_path))
    });
    prepared.dedup_by(|left, right| {
        left.logical_root == right.logical_root && left.physical_path == right.physical_path
    });
    Ok(prepared)
}

fn sort_dedup(paths: &mut Vec<PathBuf>) {
    paths.sort_unstable();
    paths.dedup();
}

fn is_sidecar_name(name: &str) -> bool {
    name.ends_with(".zmeta") || name.ends_with(".meta.toml")
}

fn sidecar_counterpart(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?.to_str()?;
    if let Some(stem) = name.strip_suffix(".meta.toml") {
        Some(path.with_file_name(format!("{stem}.zmeta")))
    } else {
        name.strip_suffix(".zmeta")
            .map(|stem| path.with_file_name(format!("{stem}.meta.toml")))
    }
}

fn is_auxiliary_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bin")
                || extension.eq_ignore_ascii_case("ttf")
                || extension.eq_ignore_ascii_case("otf")
                || extension.eq_ignore_ascii_case("woff")
                || extension.eq_ignore_ascii_case("woff2")
        })
}

fn is_supported_authoring_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.ends_with(".scene.toml") || name.ends_with(".model.toml") || name.ends_with(".zmaterial")
}

#[cfg(test)]
pub(crate) struct MigrationInventoryTestSnapshot {
    pub(crate) directory_visits: usize,
    pub(crate) directory_sorts: usize,
    pub(crate) file_visits: usize,
    pub(crate) authoring_files: Vec<PathBuf>,
    pub(crate) sidecar_candidates: Vec<PathBuf>,
    pub(crate) transaction_targets: Vec<PathBuf>,
    pub(crate) physical_relative_paths: Vec<(PathBuf, Vec<PathBuf>)>,
    pub(crate) logical_root_identities: Vec<(PathBuf, Vec<RelPath>)>,
}

#[cfg(test)]
pub(crate) fn scan_migration_inventory_for_test(
    roots: &[PathBuf],
) -> Result<MigrationInventoryTestSnapshot, std::io::Error> {
    let roots = roots
        .iter()
        .enumerate()
        .map(|(index, path)| {
            (
                RelPath::parse(format!("test-root-{index}")).expect("test root must be valid"),
                path.clone(),
            )
        })
        .collect::<Vec<_>>();
    let inventory = MigrationInventory::build(&roots)?;
    let physical_relative_paths = inventory
        .physical_identities
        .iter()
        .map(|identity| {
            (
                identity.path.clone(),
                identity
                    .root_relative_identities
                    .iter()
                    .map(|root_relative| root_relative.relative.clone())
                    .collect(),
            )
        })
        .collect();
    let logical_root_identities = inventory
        .physical_identities
        .iter()
        .map(|identity| {
            (
                identity.path.clone(),
                identity
                    .root_relative_identities
                    .iter()
                    .map(|root_relative| root_relative.logical_root.clone())
                    .collect(),
            )
        })
        .collect();
    Ok(MigrationInventoryTestSnapshot {
        directory_visits: inventory.stats.directory_visits,
        directory_sorts: inventory.stats.directory_sorts,
        file_visits: inventory.stats.file_visits,
        authoring_files: inventory.authoring_files,
        sidecar_candidates: inventory.sidecar_candidates,
        transaction_targets: inventory.transaction_targets,
        physical_relative_paths,
        logical_root_identities,
    })
}

#[cfg(test)]
#[path = "scan/optimization_tests.rs"]
mod optimization_tests;
