use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, ErrorKind, Read};
use std::path::{Path, PathBuf};

pub(super) struct NativeDynamicPackageFileEntry {
    pub(super) source_path: PathBuf,
    pub(super) relative_path: String,
}

pub(super) struct NativeDynamicPackageFileInventory {
    pub(super) entries: Vec<NativeDynamicPackageFileEntry>,
    pub(super) diagnostics: Vec<String>,
}

pub(super) fn copy_native_dynamic_package_files(
    entries: &[NativeDynamicPackageFileEntry],
    destination: &Path,
) -> Result<usize, std::io::Error> {
    let mut copied = 0;
    let mut created_parents = HashSet::new();
    fs::create_dir_all(destination)?;
    for entry in entries {
        let destination_path = destination.join(&entry.relative_path);
        if let Some(parent) = destination_path.parent() {
            if created_parents.insert(parent.to_path_buf()) {
                fs::create_dir_all(parent)?;
            }
        }
        if copy_file_if_changed(&entry.source_path, &destination_path)? {
            copied += 1;
        }
    }
    Ok(copied)
}

// Native payloads can be large, so equality is checked with bounded buffers after the cheap size
// gate. Timestamps are deliberately ignored because export roots can be restored or copied.
fn copy_file_if_changed(source: &Path, destination: &Path) -> Result<bool, std::io::Error> {
    if files_match(source, destination)? {
        return Ok(false);
    }
    fs::copy(source, destination)?;
    Ok(true)
}

fn files_match(source: &Path, destination: &Path) -> Result<bool, std::io::Error> {
    let source_metadata = fs::metadata(source)?;
    let destination_metadata = match fs::metadata(destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if source_metadata.len() != destination_metadata.len() {
        return Ok(false);
    }

    let mut source = BufReader::new(File::open(source)?);
    let mut destination = BufReader::new(File::open(destination)?);
    let mut source_buffer = [0_u8; 64 * 1024];
    let mut destination_buffer = [0_u8; 64 * 1024];
    let mut remaining = source_metadata.len();
    while remaining > 0 {
        let chunk_len = remaining.min(source_buffer.len() as u64) as usize;
        source.read_exact(&mut source_buffer[..chunk_len])?;
        destination.read_exact(&mut destination_buffer[..chunk_len])?;
        if source_buffer[..chunk_len] != destination_buffer[..chunk_len] {
            return Ok(false);
        }
        remaining -= chunk_len as u64;
    }
    Ok(true)
}

pub(super) fn native_dynamic_package_file_inventory(
    source: &Path,
    package_id: &str,
) -> Result<NativeDynamicPackageFileInventory, std::io::Error> {
    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    let mut saw_native_dir = false;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            diagnostics.push(format!(
                "native dynamic package {package_id} skipped symlinked payload {}",
                entry.path().display()
            ));
            continue;
        }

        let source_path = entry.path();
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if file_type.is_dir() {
            if file_name == "native" {
                saw_native_dir = true;
                let previous_entry_count = entries.len();
                collect_native_artifact_entries(&source_path, file_name, &mut entries)?;
                if entries.len() == previous_entry_count {
                    diagnostics.push(format!(
                        "native dynamic package {package_id} has no dynamic library artifacts under {}",
                        source_path.display()
                    ));
                }
            } else if should_copy_native_resource_dir(file_name) {
                collect_resource_entries(&source_path, file_name, &mut entries)?;
            }
        } else if should_copy_native_dynamic_file(file_name) {
            entries.push(NativeDynamicPackageFileEntry {
                source_path,
                relative_path: file_name.to_string(),
            });
        }
    }
    if !saw_native_dir {
        diagnostics.push(format!(
            "native dynamic package {package_id} has no native artifact directory under {}",
            source.display()
        ));
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(NativeDynamicPackageFileInventory {
        entries,
        diagnostics,
    })
}

fn should_copy_native_resource_dir(name: &str) -> bool {
    matches!(name, "assets" | "asset" | "resources" | "resource")
}

fn should_copy_native_dynamic_file(name: &str) -> bool {
    name == "plugin.toml"
}

fn collect_resource_entries(
    source: &Path,
    relative_prefix: &str,
    entries: &mut Vec<NativeDynamicPackageFileEntry>,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let source_path = entry.path();
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let relative_path = format!("{relative_prefix}/{file_name}");
        if file_type.is_dir() {
            collect_resource_entries(&source_path, &relative_path, entries)?;
        } else {
            entries.push(NativeDynamicPackageFileEntry {
                source_path,
                relative_path,
            });
        }
    }
    Ok(())
}

fn collect_native_artifact_entries(
    source: &Path,
    relative_prefix: &str,
    entries: &mut Vec<NativeDynamicPackageFileEntry>,
) -> Result<(), std::io::Error> {
    if !is_real_directory(source)? {
        return Ok(());
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || file_type.is_dir() {
            continue;
        }
        let source_path = entry.path();
        if !is_native_dynamic_artifact(&source_path) {
            continue;
        }
        let Some(file_name) = source_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        entries.push(NativeDynamicPackageFileEntry {
            source_path,
            relative_path: format!("{relative_prefix}/{file_name}"),
        });
    }
    Ok(())
}

fn is_real_directory(path: &Path) -> Result<bool, std::io::Error> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir() && !metadata.file_type().is_symlink()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn is_native_dynamic_artifact(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    match extension.len() {
        2 => extension.eq_ignore_ascii_case("so"),
        3 => ["dll", "pdb", "dbg"]
            .iter()
            .any(|supported| extension.eq_ignore_ascii_case(supported)),
        4 => extension.eq_ignore_ascii_case("dsym"),
        5 => extension.eq_ignore_ascii_case("dylib"),
        _ => false,
    }
}

#[cfg(test)]
#[path = "copy/native_extension_tests.rs"]
mod native_extension_tests;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::copy_file_if_changed;

    #[test]
    fn native_file_copy_skips_equal_contents_and_replaces_changed_contents() {
        let root = temporary_test_root();
        fs::create_dir_all(&root).expect("test root should be created");
        let source = root.join("source.dll");
        let destination = root.join("destination.dll");
        fs::write(&source, "stable").expect("source fixture should be written");
        fs::write(&destination, "stable").expect("destination fixture should be written");

        let original_permissions = fs::metadata(&destination)
            .expect("destination metadata should be readable")
            .permissions();
        let mut read_only_permissions = original_permissions.clone();
        read_only_permissions.set_readonly(true);
        fs::set_permissions(&destination, read_only_permissions)
            .expect("destination should become read-only");

        assert!(!copy_file_if_changed(&source, &destination)
            .expect("equal native contents should not rewrite the destination"));

        fs::set_permissions(&destination, original_permissions)
            .expect("destination should become writable again");
        fs::write(&source, "changed").expect("source fixture should be updated");

        assert!(copy_file_if_changed(&source, &destination)
            .expect("changed native contents should replace the destination"));
        assert_eq!(
            fs::read_to_string(&destination).expect("destination should be readable"),
            "changed"
        );

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    #[test]
    fn native_file_copy_compares_every_bounded_buffer_chunk() {
        let root = temporary_test_root();
        fs::create_dir_all(&root).expect("test root should be created");
        let source = root.join("source.pdb");
        let destination = root.join("destination.pdb");
        let mut payload = vec![7_u8; 64 * 1024 + 3];
        fs::write(&source, &payload).expect("source fixture should be written");
        fs::write(&destination, &payload).expect("destination fixture should be written");

        assert!(!copy_file_if_changed(&source, &destination)
            .expect("equal multi-chunk native contents should be skipped"));

        *payload
            .last_mut()
            .expect("multi-chunk fixture should have a trailing byte") = 9;
        fs::write(&source, &payload).expect("source fixture should be updated");
        assert!(copy_file_if_changed(&source, &destination)
            .expect("a trailing chunk change should replace the destination"));
        assert_eq!(
            fs::read(&destination).expect("destination should be readable"),
            payload
        );

        fs::remove_dir_all(root).expect("test root should be removable");
    }

    fn temporary_test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zircon-export-native-incremental-{}-{nonce}",
            std::process::id()
        ))
    }
}
