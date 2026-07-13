use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::project::{AssetMetaDocument, AssetSourceUnit};
use crate::asset::{AssetImportError, AssetUri};

use super::super::{
    collect_files::collect_files, meta_path_for_source::meta_path_for_source,
    source_mtime_unix_ms::source_mtime_unix_ms, ProjectManager,
};

pub(super) struct AssetImportSource {
    pub(super) path: PathBuf,
    pub(super) uri: AssetUri,
    pub(super) meta_path: PathBuf,
    pub(super) unit: AssetSourceUnit,
    pub(super) included_files: Vec<AssetUri>,
    pub(super) included_paths: Vec<PathBuf>,
    pub(super) compound_root: Option<PathBuf>,
}

impl ProjectManager {
    pub(super) fn collect_import_sources(
        &self,
    ) -> Result<Vec<AssetImportSource>, AssetImportError> {
        let mut sources = Vec::new();
        let project_roots = self.package_assets.project_roots().to_vec();
        for root in &project_roots {
            self.collect_import_sources_for_root(root, None, &mut sources)?;
        }

        for (package_id, root) in self.package_assets.iter() {
            self.collect_import_sources_for_root(root, Some(package_id), &mut sources)?;
        }

        reject_duplicate_project_uris(&sources)?;
        sources.sort_by(|left, right| left.uri.cmp(&right.uri));
        Ok(sources)
    }

    fn collect_import_sources_for_root(
        &self,
        root: &Path,
        package_id: Option<&str>,
        sources: &mut Vec<AssetImportSource>,
    ) -> Result<(), AssetImportError> {
        let mut compound_sources = self.collect_compound_sources_for_root(root, package_id)?;
        let compound_roots = compound_sources
            .iter()
            .filter_map(|source| source.compound_root.clone())
            .collect::<Vec<_>>();

        let mut files = Vec::new();
        collect_files(root, &mut files)?;
        for file in files {
            if compound_roots
                .iter()
                .any(|compound_root| file.starts_with(compound_root))
            {
                continue;
            }
            sources.push(AssetImportSource {
                uri: self.source_uri_for_asset_root_path(root, package_id, &file)?,
                path: file.clone(),
                meta_path: meta_path_for_source(&file),
                unit: AssetSourceUnit::Single,
                included_files: Vec::new(),
                included_paths: Vec::new(),
                compound_root: None,
            });
        }

        sources.append(&mut compound_sources);
        Ok(())
    }

    fn collect_compound_sources_for_root(
        &self,
        root: &Path,
        package_id: Option<&str>,
    ) -> Result<Vec<AssetImportSource>, AssetImportError> {
        let mut meta_files = Vec::new();
        collect_zmeta_files(root, &mut meta_files)?;
        let mut sources = Vec::new();

        for meta_path in meta_files {
            let Ok(meta) = AssetMetaDocument::load(&meta_path) else {
                continue;
            };
            if meta.unit != AssetSourceUnit::Compound {
                continue;
            }
            let Some(compound_root) = compound_root_for_meta_path(&meta_path) else {
                continue;
            };
            let mut included_paths = Vec::new();
            collect_files(&compound_root, &mut included_paths)?;
            included_paths.sort();
            let included_files = included_paths
                .iter()
                .map(|path| self.source_uri_for_asset_root_path(root, package_id, path))
                .collect::<Result<Vec<_>, _>>()?;
            sources.push(AssetImportSource {
                uri: self.source_uri_for_asset_root_path(root, package_id, &compound_root)?,
                path: meta_path.clone(),
                meta_path,
                unit: AssetSourceUnit::Compound,
                included_files,
                included_paths,
                compound_root: Some(compound_root),
            });
        }

        Ok(sources)
    }

    fn source_uri_for_asset_root_path(
        &self,
        root: &Path,
        package_id: Option<&str>,
        path: &Path,
    ) -> Result<AssetUri, AssetImportError> {
        if let Some(package_id) = package_id {
            self.source_uri_for_package_path(package_id, root, path)
        } else {
            self.source_uri_for_path(root, path)
        }
    }
}

fn reject_duplicate_project_uris(sources: &[AssetImportSource]) -> Result<(), AssetImportError> {
    let mut paths_by_uri = BTreeMap::new();
    for source in sources {
        if source.uri.package_id().is_some() {
            continue;
        }
        if let Some(previous) = paths_by_uri.insert(source.uri.clone(), source.path.clone()) {
            return Err(AssetImportError::DuplicateProjectAssetUri {
                uri: source.uri.clone(),
                first: previous,
                second: source.path.clone(),
            });
        }
    }
    Ok(())
}

fn collect_zmeta_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_zmeta_files(&path, files)?;
        } else if path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .is_some_and(|file_name| file_name.ends_with(".zmeta"))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn compound_root_for_meta_path(meta_path: &Path) -> Option<PathBuf> {
    let file_name = meta_path.file_name()?.to_str()?;
    let root_name = file_name.strip_suffix(".zmeta")?;
    Some(meta_path.with_file_name(root_name))
}

pub(super) fn source_bytes_for_import(
    source: &AssetImportSource,
) -> Result<Vec<u8>, AssetImportError> {
    let mut bytes = fs::read(&source.path)?;
    let Some(compound_root) = &source.compound_root else {
        return Ok(bytes);
    };

    for included_path in &source.included_paths {
        let relative = included_path
            .strip_prefix(compound_root)
            .unwrap_or(included_path.as_path());
        bytes.extend_from_slice(b"\n# included ");
        bytes.extend_from_slice(relative.to_string_lossy().as_bytes());
        bytes.extend_from_slice(b"\n");
        bytes.extend_from_slice(&fs::read(included_path)?);
    }
    Ok(bytes)
}

pub(super) fn source_mtime_unix_ms_for_import(
    source: &AssetImportSource,
) -> Result<u64, AssetImportError> {
    let mut mtime = source_mtime_unix_ms(&source.path)?;
    for included_path in &source.included_paths {
        mtime = mtime.max(source_mtime_unix_ms(included_path)?);
    }
    Ok(mtime)
}
