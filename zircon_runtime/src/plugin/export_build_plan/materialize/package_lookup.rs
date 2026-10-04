use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::plugin::PluginPackageManifest;

use super::copy::{native_dynamic_package_file_inventory, NativeDynamicPackageFileInventory};

/// Immutable native package lookup for one export materialization generation.
///
/// It resolves selected package ids and their exportable payload entries once, never follows
/// symlinks, and selects the direct `<plugin-root>/<package-id>` directory before a deterministic
/// lexical nested fallback.
pub(super) struct NativePackageInventory {
    plugin_root: PathBuf,
    package_dirs: BTreeMap<String, PathBuf>,
    file_inventories: BTreeMap<String, NativeDynamicPackageFileInventory>,
}

impl NativePackageInventory {
    pub(super) fn build(
        plugin_root: &Path,
        selected_package_ids: &[String],
    ) -> Result<Self, std::io::Error> {
        let selected_package_ids = selected_package_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if selected_package_ids.is_empty() {
            return Ok(Self {
                plugin_root: plugin_root.to_path_buf(),
                package_dirs: BTreeMap::new(),
                file_inventories: BTreeMap::new(),
            });
        }
        if !is_real_directory(plugin_root)? {
            return Ok(Self {
                plugin_root: plugin_root.to_path_buf(),
                package_dirs: BTreeMap::new(),
                file_inventories: BTreeMap::new(),
            });
        }

        let mut package_dirs = BTreeMap::new();
        let mut unresolved_package_ids =
            selected_package_ids.iter().copied().collect::<HashSet<_>>();
        for package_id in &selected_package_ids {
            let Some(package_dir) = direct_child_package_dir(plugin_root, package_id) else {
                continue;
            };
            if !is_real_directory(&package_dir)? {
                continue;
            }
            if let Some(manifest) = package_manifest(&package_dir.join("plugin.toml"))? {
                if manifest.id != *package_id {
                    continue;
                }
                require_product_role(&manifest)?;
                package_dirs.insert((*package_id).to_owned(), package_dir);
                unresolved_package_ids.remove(*package_id);
            }
        }

        if unresolved_package_ids.is_empty() {
            return Self::finish(plugin_root, package_dirs);
        }

        let mut resolved_package_dirs = package_dirs.values().cloned().collect::<HashSet<_>>();
        let mut stack = vec![plugin_root.to_path_buf()];
        'search: while let Some(current) = stack.pop() {
            let mut entries = fs::read_dir(&current)?.collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries.into_iter().rev() {
                let file_type = entry.file_type()?;
                if file_type.is_symlink() || !file_type.is_dir() {
                    continue;
                }
                let package_dir = entry.path();
                if resolved_package_dirs.contains(&package_dir) {
                    stack.push(package_dir);
                    continue;
                }
                if let Some(manifest) = package_manifest(&package_dir.join("plugin.toml"))? {
                    let package_id = manifest.id.as_str();
                    if selected_package_ids.contains(package_id) {
                        if unresolved_package_ids.contains(package_id) {
                            require_product_role(&manifest)?;
                            unresolved_package_ids.remove(package_id);
                            package_dirs.insert(package_id.to_owned(), package_dir.clone());
                            resolved_package_dirs.insert(package_dir);
                            if unresolved_package_ids.is_empty() {
                                break 'search;
                            }
                        }
                        continue;
                    }
                }
                stack.push(package_dir);
            }
        }

        Self::finish(plugin_root, package_dirs)
    }

    pub(super) fn package_dir(&self, package_id: &str) -> Option<&Path> {
        self.package_dirs.get(package_id).map(PathBuf::as_path)
    }

    pub(super) fn file_inventory(
        &self,
        package_id: &str,
    ) -> Option<&NativeDynamicPackageFileInventory> {
        self.file_inventories.get(package_id)
    }

    pub(super) fn plugin_root(&self) -> &Path {
        &self.plugin_root
    }

    fn finish(
        plugin_root: &Path,
        package_dirs: BTreeMap<String, PathBuf>,
    ) -> Result<Self, std::io::Error> {
        let mut file_inventories = BTreeMap::new();
        for (package_id, package_dir) in &package_dirs {
            file_inventories.insert(
                package_id.clone(),
                native_dynamic_package_file_inventory(package_dir, package_id)?,
            );
        }
        Ok(Self {
            plugin_root: plugin_root.to_path_buf(),
            package_dirs,
            file_inventories,
        })
    }
}

fn direct_child_package_dir(plugin_root: &Path, package_id: &str) -> Option<PathBuf> {
    let mut components = Path::new(package_id).components();
    let Some(Component::Normal(_)) = components.next() else {
        return None;
    };
    (components.next().is_none()).then(|| plugin_root.join(package_id))
}

fn is_real_directory(path: &Path) -> Result<bool, std::io::Error> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir() && !metadata.file_type().is_symlink()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn package_manifest(path: &Path) -> Result<Option<PluginPackageManifest>, std::io::Error> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(None);
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Ok(None);
    }

    let source = fs::read_to_string(path)?;
    Ok(toml::from_str(&source).ok())
}

fn require_product_role(manifest: &PluginPackageManifest) -> std::io::Result<()> {
    if !manifest.package_role.is_product_catalog_eligible() {
        // Never fall back to another same-ID directory after finding an ineligible selection.
        return Err(std::io::Error::new(
            ErrorKind::PermissionDenied,
            format!(
                "selected native package {} with role {:?} is not eligible for product export",
                manifest.id, manifest.package_role
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/package_lookup.rs"]
mod tests;
