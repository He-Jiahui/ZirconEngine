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
            if is_real_directory(&package_dir)?
                && package_manifest_id(&package_dir.join("plugin.toml"))?.as_deref()
                    == Some(*package_id)
            {
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
                if let Some(package_id) = package_manifest_id(&package_dir.join("plugin.toml"))? {
                    if selected_package_ids.contains(package_id.as_str()) {
                        if unresolved_package_ids.remove(package_id.as_str()) {
                            package_dirs.insert(package_id, package_dir.clone());
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

fn package_manifest_id(path: &Path) -> Result<Option<String>, std::io::Error> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(None);
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Ok(None);
    }

    let source = fs::read_to_string(path)?;
    Ok(toml::from_str::<PluginPackageManifest>(&source)
        .ok()
        .map(|manifest| manifest.id))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::NativePackageInventory;

    #[test]
    fn inventory_prefers_direct_packages_and_reuses_nested_manifest_scan() {
        let root = temporary_test_root();
        let direct_rendering = root.join("rendering");
        let nested_rendering = root.join("aliases").join("rendering");
        let nested_audio = root.join("third_party").join("audio");
        let lexical_fallback_audio = root.join("vendor").join("audio");
        write_plugin_manifest(&direct_rendering, "rendering");
        write_plugin_manifest(&nested_rendering, "rendering");
        write_plugin_manifest(&nested_audio, "audio");
        write_plugin_manifest(&lexical_fallback_audio, "audio");

        let selected_package_ids = vec!["rendering".to_owned(), "audio".to_owned()];
        let inventory = NativePackageInventory::build(&root, &selected_package_ids)
            .expect("inventory should scan once");

        assert_eq!(
            inventory.package_dir("rendering"),
            Some(direct_rendering.as_path())
        );
        assert_eq!(inventory.package_dir("audio"), Some(nested_audio.as_path()));
        assert_eq!(inventory.package_dir("missing"), None);

        fs::remove_file(nested_audio.join("plugin.toml"))
            .expect("fixture manifest should be removable after inventory build");
        assert_eq!(inventory.package_dir("audio"), Some(nested_audio.as_path()));

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn inventory_skips_unrelated_manifest_errors_when_direct_selection_resolves() {
        let root = temporary_test_root();
        let direct_rendering = root.join("rendering");
        write_plugin_manifest(&direct_rendering, "rendering");
        let unrelated_manifest = root.join("third_party").join("plugin.toml");
        fs::create_dir_all(
            unrelated_manifest
                .parent()
                .expect("fixture parent should exist"),
        )
        .expect("fixture directory should be created");
        fs::write(&unrelated_manifest, [0xff]).expect("fixture invalid manifest should be written");

        let selected_package_ids = vec!["rendering".to_owned()];
        let inventory = NativePackageInventory::build(&root, &selected_package_ids)
            .expect("direct selection should not read unrelated manifests");

        assert_eq!(
            inventory.package_dir("rendering"),
            Some(direct_rendering.as_path())
        );

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn inventory_stops_after_all_nested_selections_resolve() {
        let root = temporary_test_root();
        let nested_audio = root.join("a").join("audio");
        let nested_rendering = root.join("b").join("rendering");
        write_plugin_manifest(&nested_audio, "audio");
        write_plugin_manifest(&nested_rendering, "rendering");
        let unrelated_manifest = root.join("z_unrelated").join("plugin.toml");
        fs::create_dir_all(
            unrelated_manifest
                .parent()
                .expect("fixture parent should exist"),
        )
        .expect("fixture directory should be created");
        fs::write(&unrelated_manifest, [0xff]).expect("fixture invalid manifest should be written");

        let selected_package_ids = vec!["audio".to_owned(), "rendering".to_owned()];
        let inventory = NativePackageInventory::build(&root, &selected_package_ids)
            .expect("resolved selections should stop the remaining tree scan");

        assert_eq!(inventory.package_dir("audio"), Some(nested_audio.as_path()));
        assert_eq!(
            inventory.package_dir("rendering"),
            Some(nested_rendering.as_path())
        );

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn inventory_searches_inside_direct_selected_packages_for_nested_selections() {
        let root = temporary_test_root();
        let direct_rendering = root.join("rendering");
        let nested_audio = direct_rendering.join("vendor").join("audio");
        write_plugin_manifest(&direct_rendering, "rendering");
        write_plugin_manifest(&nested_audio, "audio");

        let selected_package_ids = vec!["rendering".to_owned(), "audio".to_owned()];
        let inventory = NativePackageInventory::build(&root, &selected_package_ids)
            .expect("nested selections inside direct packages should remain searchable");

        assert_eq!(
            inventory.package_dir("rendering"),
            Some(direct_rendering.as_path())
        );
        assert_eq!(inventory.package_dir("audio"), Some(nested_audio.as_path()));

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn inventory_snapshots_selected_native_payload_entries() {
        let root = temporary_test_root();
        let package_dir = root.join("audio");
        write_plugin_manifest(&package_dir, "audio");
        let native_artifact = package_dir.join("native").join("audio.dll");
        let resource = package_dir.join("assets").join("settings.json");
        fs::create_dir_all(
            native_artifact
                .parent()
                .expect("native fixture parent should exist"),
        )
        .expect("native fixture directory should be created");
        fs::create_dir_all(
            resource
                .parent()
                .expect("resource fixture parent should exist"),
        )
        .expect("resource fixture directory should be created");
        fs::write(&native_artifact, "native payload").expect("native fixture should be written");
        fs::write(&resource, "resource payload").expect("resource fixture should be written");

        let selected_package_ids = vec!["audio".to_owned()];
        let inventory = NativePackageInventory::build(&root, &selected_package_ids)
            .expect("inventory should snapshot the selected package payload");

        fs::remove_file(&native_artifact)
            .expect("source artifact should be removable after inventory build");
        let payload = inventory
            .file_inventory("audio")
            .expect("selected package should retain its frozen payload inventory");
        let relative_paths = payload
            .entries
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(relative_paths, ["assets/settings.json", "native/audio.dll"]);

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn hash_resolution_sets_preserve_lexical_fallback_order() {
        let root = temporary_test_root();
        let lexical_first = root.join("a").join("audio");
        let lexical_last = root.join("z").join("audio");
        write_plugin_manifest(&lexical_first, "audio");
        write_plugin_manifest(&lexical_last, "audio");

        let inventory = NativePackageInventory::build(&root, &["audio".to_owned()])
            .expect("nested selection should resolve");

        assert_eq!(
            inventory.package_dir("audio"),
            Some(lexical_first.as_path())
        );

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn hash_resolution_sets_deduplicate_selected_package_ids() {
        let root = temporary_test_root();
        let direct_audio = root.join("audio");
        write_plugin_manifest(&direct_audio, "audio");

        let inventory = NativePackageInventory::build(
            &root,
            &["audio".to_owned(), "audio".to_owned(), "audio".to_owned()],
        )
        .expect("duplicate selections should resolve once");

        assert_eq!(inventory.package_dirs.len(), 1);
        assert_eq!(inventory.package_dir("audio"), Some(direct_audio.as_path()));

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    fn temporary_test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zircon-export-native-package-inventory-{}-{nonce}",
            std::process::id()
        ))
    }

    fn write_plugin_manifest(directory: &Path, id: &str) {
        fs::create_dir_all(directory).expect("fixture directory should be created");
        fs::write(
            directory.join("plugin.toml"),
            format!("id = {id:?}\nversion = \"0.1.0\"\ndisplay_name = {id:?}\n"),
        )
        .expect("fixture plugin manifest should be written");
    }
}
