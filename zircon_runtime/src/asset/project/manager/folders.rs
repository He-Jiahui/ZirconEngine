//! Folder mutations reuse root-source relocation preparation and publish one project generation.

use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::asset::safe_mutation_path::{
    create_directory, read_directory, read_directory_entries, remove_empty_directory,
    validate_no_follow_path, DirectoryMutationOutcome, MAX_DIRECTORY_ENTRIES,
};
use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::{AssetImportError, AssetUri};
use crate::core::resource::{ResourceRecord, ResourceScheme};

use super::durable_transaction::{
    journal_directory, PreparedFileWrite, ProjectFileCommitOutcome, ProjectTransactionFault,
};
use super::meta_path_for_source::meta_path_for_source;
use super::{PreparedProjectSourceRelocation, ProjectManager};

#[derive(Debug, PartialEq, Eq)]
struct FolderTree {
    directories: Vec<PathBuf>,
    files: BTreeSet<PathBuf>,
}

pub(crate) struct PreparedProjectFolderRelocation {
    relocation: PreparedProjectSourceRelocation,
    project_root: PathBuf,
    source_path: PathBuf,
    target_path: PathBuf,
    expected_tree: FolderTree,
    no_change: bool,
}

pub(crate) struct ProjectFolderCommitOutcome {
    files: ProjectFileCommitOutcome,
    directory_cleanup: io::Result<()>,
}

impl ProjectFolderCommitOutcome {
    pub(crate) fn ensure_durable(self) -> Result<(), AssetImportError> {
        self.files.ensure_durable()?;
        self.directory_cleanup.map_err(Into::into)
    }
}

impl PreparedProjectFolderRelocation {
    pub(crate) fn relocation(&self) -> &PreparedProjectSourceRelocation {
        &self.relocation
    }

    pub(crate) fn no_change(&self) -> bool {
        self.no_change
    }

    pub(crate) fn commit(self) -> Result<ProjectFolderCommitOutcome, AssetImportError> {
        self.commit_internal(ProjectTransactionFault::None)
    }

    #[cfg(test)]
    pub(crate) fn commit_with_fault(
        self,
        fault: ProjectTransactionFault,
    ) -> Result<ProjectFolderCommitOutcome, AssetImportError> {
        self.commit_internal(fault)
    }

    fn commit_internal(
        self,
        fault: ProjectTransactionFault,
    ) -> Result<ProjectFolderCommitOutcome, AssetImportError> {
        let current = inspect_tree(&self.project_root, &self.source_path)?;
        if current != self.expected_tree {
            return Err(folder_error(
                "folder contents changed while the rename was prepared",
            ));
        }
        if self.no_change {
            return Ok(ProjectFolderCommitOutcome {
                files: ProjectFileCommitOutcome::Durable,
                directory_cleanup: Ok(()),
            });
        }
        validate_no_follow_path(&self.project_root, &self.target_path, true)?;
        ensure_missing(&self.target_path)?;
        let mut created = Vec::new();
        for relative in &self.expected_tree.directories {
            let destination = self.target_path.join(relative);
            let outcome = match create_directory(&self.project_root, &destination) {
                Ok(outcome) => outcome,
                Err(error) => {
                    remove_created_directories(&self.project_root, &created);
                    return Err(error.into());
                }
            };
            created.push(destination);
            if let Err(error) = outcome.ensure_durable() {
                remove_created_directories(&self.project_root, &created);
                return Err(error.into());
            }
        }
        #[cfg(test)]
        let file_commit = self.relocation.commit_with_fault(fault);
        #[cfg(not(test))]
        let file_commit = {
            let _ = fault;
            self.relocation.commit()
        };
        let files = match file_commit {
            Ok(files) => files,
            Err(error) => {
                remove_created_directories(&self.project_root, &created);
                return Err(error);
            }
        };
        // File publication is already committed. A concurrent new file can prevent removing an
        // old directory; report that after installing the candidate, never discard the new state.
        let directory_cleanup =
            self.expected_tree
                .directories
                .iter()
                .rev()
                .try_for_each(|relative| {
                    remove_empty_directory(&self.project_root, &self.source_path.join(relative))
                        .and_then(DirectoryMutationOutcome::ensure_durable)
                });
        Ok(ProjectFolderCommitOutcome {
            files,
            directory_cleanup,
        })
    }
}

impl ProjectManager {
    /// Physical project directories, including empty ones, for the editor catalog generation.
    /// The `res://` root is implicit; package directories have their own catalog owner.
    pub fn project_folder_uris(&self) -> Result<Vec<AssetUri>, AssetImportError> {
        self.project_folder_uris_with_limit(MAX_DIRECTORY_ENTRIES)
    }

    fn project_folder_uris_with_limit(
        &self,
        limit: usize,
    ) -> Result<Vec<AssetUri>, AssetImportError> {
        let mut folders = BTreeSet::new();
        let mut visited_folders = 0;
        for root in self.project_asset_roots() {
            let mut pending = vec![PathBuf::new()];
            while let Some(relative) = pending.pop() {
                for (name, metadata) in
                    read_directory_entries(self.paths.root(), &root.join(&relative))?
                {
                    if !metadata.is_dir() || is_link_or_reparse(&metadata) {
                        continue;
                    }
                    let child = relative.join(name);
                    if validate_no_follow_path(self.paths.root(), &root.join(&child), false)
                        .is_err()
                    {
                        continue;
                    }
                    // Some OS names cannot be represented by the asset locator grammar.
                    // They are outside the browser inventory, and must not hide valid folders.
                    if let Ok(folder) = folder_uri_for_relative(&child) {
                        admit_tree_entry(&mut visited_folders, limit)?;
                        folders.insert(folder);
                        pending.push(child);
                    }
                }
            }
        }
        Ok(folders.into_iter().collect())
    }

    pub(crate) fn create_project_folder(
        &mut self,
        folder: &AssetUri,
    ) -> Result<DirectoryMutationOutcome, AssetImportError> {
        let target = self.missing_project_folder_target(folder)?;
        let outcome = create_directory(self.paths.root(), &target)?;
        self.advance_folder_catalog_generation();
        Ok(outcome)
    }

    /// Folder deletion never recursively removes contents, including unimported files.
    pub(crate) fn delete_project_folder(
        &mut self,
        folder: &AssetUri,
    ) -> Result<DirectoryMutationOutcome, AssetImportError> {
        let source = self.existing_project_folder(folder)?;
        let outcome = remove_empty_directory(self.paths.root(), &source)?;
        self.advance_folder_catalog_generation();
        Ok(outcome)
    }

    pub(crate) fn prepare_project_folder_relocation(
        &mut self,
        source: AssetUri,
        target: AssetUri,
    ) -> Result<PreparedProjectFolderRelocation, AssetImportError> {
        validate_folder_uri(&source)?;
        validate_folder_uri(&target)?;
        let source_path = self.existing_project_folder(&source)?;
        if source != target && (is_descendant(&source, &target) || is_descendant(&target, &source))
        {
            return Err(folder_error(
                "folder rename cannot overlap its source or target subtree",
            ));
        }
        let no_change = source == target;
        let target_path = if no_change {
            source_path.clone()
        } else {
            self.missing_project_folder_target(&target)?
        };
        let expected_tree = inspect_tree(self.paths.root(), &source_path)?;
        let mut relocations = Vec::new();
        if !no_change {
            if is_descendant(&source, &self.manifest.default_scene) {
                return Err(folder_error(
                    "folder rename cannot relocate the active default scene",
                ));
            }
            if self
                .manifest
                .ui_roots
                .iter()
                .any(|root| is_descendant(&source, root))
            {
                return Err(folder_error(
                    "folder rename cannot relocate a manifest UI root",
                ));
            }
            let source_entries = self
                .asset_registry
                .entries_iter()
                .filter(|entry| {
                    entry.path().label().is_none() && is_descendant(&source, entry.path())
                })
                .map(|entry| (entry.uuid(), entry.path().clone()))
                .collect::<Vec<_>>();
            let mut tracked_files = BTreeSet::new();
            for (_, locator) in &source_entries {
                let relative = descendant_relative_path(&source, locator)?;
                tracked_files.insert(relative.clone());
                tracked_files.insert(meta_path_for_source(&relative));
                if self.source_path_for_no_follow(locator)? != source_path.join(&relative) {
                    return Err(folder_error(
                        "folder rename source has an ambiguous physical asset root",
                    ));
                }
            }
            if let Some(untracked) = expected_tree.files.difference(&tracked_files).next() {
                return Err(folder_error(format!(
                    "folder rename refuses an untracked or unsupported file: {}",
                    source_path.join(untracked).display()
                )));
            }
            for (uuid, locator) in source_entries {
                let relative = descendant_relative_path(&source, &locator)?;
                let destination = AssetUri::new(
                    ResourceScheme::Res,
                    format!("{}/{}", target.path(), relative_uri_path(&relative)?),
                    None,
                )?;
                relocations.push(self.prepare_project_source_relocation(uuid, destination)?);
            }
        }
        let affected_ids = relocations
            .iter()
            .flat_map(|relocation| relocation.updated_records().iter().map(ResourceRecord::id))
            .collect::<HashSet<_>>();
        let mut updated = affected_ids
            .into_iter()
            .filter_map(|id| self.registry.get(id).cloned())
            .collect::<Vec<_>>();
        updated.sort_by(|left, right| left.primary_locator().cmp(right.primary_locator()));
        if !no_change && relocations.is_empty() {
            self.advance_folder_catalog_generation();
        }
        let persisted = self
            .asset_registry
            .prepare_persistence(self.paths.registry_root())?;
        let relocation = PreparedProjectSourceRelocation::combine(
            self.paths.root().to_path_buf(),
            source,
            target,
            journal_directory(&self.paths),
            relocations,
            PreparedFileWrite::new(persisted.path, persisted.bytes),
            updated,
        );
        Ok(PreparedProjectFolderRelocation {
            relocation,
            project_root: self.paths.root().to_path_buf(),
            source_path,
            target_path,
            expected_tree,
            no_change,
        })
    }

    fn existing_project_folder(&self, folder: &AssetUri) -> Result<PathBuf, AssetImportError> {
        validate_folder_uri(folder)?;
        let source = self.source_path_for_no_follow(folder)?;
        validate_no_follow_path(self.paths.root(), &source, false)?;
        read_directory(self.paths.root(), &source)?;
        Ok(source)
    }

    fn advance_folder_catalog_generation(&mut self) {
        self.catalog_input_generation =
            crate::asset::project::ProjectCatalogInputGeneration::publish_folder_topology(
                &self.catalog_input_generation,
            );
    }

    fn missing_project_folder_target(
        &self,
        folder: &AssetUri,
    ) -> Result<PathBuf, AssetImportError> {
        validate_folder_uri(folder)?;
        // Empty directories in secondary roots still occupy the same logical res:// identity.
        for root in self.project_asset_roots() {
            let candidate = root.join(folder.path());
            validate_no_follow_path(self.paths.root(), &candidate, true)?;
            ensure_missing(&candidate)?;
        }
        let target = self.primary_project_source_path_for_no_follow(folder)?;
        let parent = target
            .parent()
            .ok_or_else(|| folder_error("folder target has no parent"))?;
        read_directory_entries(self.paths.root(), parent)?;
        Ok(target)
    }
}

fn validate_folder_uri(folder: &AssetUri) -> Result<(), AssetImportError> {
    if folder.scheme() != ResourceScheme::Res
        || folder.label().is_some()
        || folder.path().is_empty()
        || Path::new(folder.path())
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || AssetUri::parse(&folder.to_string()).as_ref() != Ok(folder)
    {
        return Err(folder_error(
            "folder operations require a nonroot res:// folder without traversal",
        ));
    }
    Ok(())
}

fn is_descendant(folder: &AssetUri, asset: &AssetUri) -> bool {
    asset.scheme() == ResourceScheme::Res
        && asset
            .path()
            .strip_prefix(folder.path())
            .is_some_and(|tail| tail.starts_with('/'))
}

fn descendant_relative_path(
    folder: &AssetUri,
    asset: &AssetUri,
) -> Result<PathBuf, AssetImportError> {
    asset
        .path()
        .strip_prefix(folder.path())
        .and_then(|tail| tail.strip_prefix('/'))
        .map(PathBuf::from)
        .ok_or_else(|| folder_error("folder relocation source is not in the requested subtree"))
}

fn inspect_tree(root: &Path, source: &Path) -> io::Result<FolderTree> {
    inspect_tree_with_limit(root, source, MAX_DIRECTORY_ENTRIES)
}

fn inspect_tree_with_limit(root: &Path, source: &Path, limit: usize) -> io::Result<FolderTree> {
    validate_no_follow_path(root, source, false)?;
    let mut tree = FolderTree {
        directories: Vec::new(),
        files: BTreeSet::new(),
    };
    let mut pending = vec![PathBuf::new()];
    let mut visited_entries = 0;
    admit_tree_entry(&mut visited_entries, limit)?;
    while let Some(relative) = pending.pop() {
        let directory = source.join(&relative);
        let mut entries = read_directory(root, &directory)?;
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        tree.directories.push(relative.clone());
        for (name, metadata) in entries.into_iter().rev() {
            admit_tree_entry(&mut visited_entries, limit)?;
            let child = relative.join(name);
            if metadata.is_dir() {
                pending.push(child);
            } else if metadata.is_file() {
                tree.files.insert(child);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "folder operations refuse a nonregular entry: {}",
                        source.join(child).display()
                    ),
                ));
            }
        }
    }
    Ok(tree)
}

fn admit_tree_entry(count: &mut usize, limit: usize) -> io::Result<()> {
    if *count >= limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("folder operation exceeds the {limit}-entry directory inventory limit"),
        ));
    }
    *count += 1;
    Ok(())
}

fn folder_uri_for_relative(relative: &Path) -> Result<AssetUri, AssetImportError> {
    let path = relative_uri_path(relative)?;
    let folder = AssetUri::new(ResourceScheme::Res, path.clone(), None)?;
    if folder.path() != path || AssetUri::parse(&folder.to_string())? != folder {
        return Err(folder_error(
            "folder path cannot round-trip as a project locator",
        ));
    }
    Ok(folder)
}

fn relative_uri_path(relative: &Path) -> Result<String, AssetImportError> {
    relative
        .components()
        .map(|part| {
            part.as_os_str()
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| folder_error("folder path is not valid UTF-8"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("/"))
}

fn ensure_missing(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("folder target is already occupied: {}", path.display()),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_created_directories(root: &Path, paths: &[PathBuf]) {
    for path in paths.iter().rev() {
        let _ = remove_empty_directory(root, path);
    }
}

fn folder_error(message: impl Into<String>) -> AssetImportError {
    AssetImportError::Parse(message.into())
}

#[cfg(test)]
#[path = "tests/folder_bounds.rs"]
mod tests;
