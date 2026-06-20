mod capture;
mod load;
mod merge;
mod mutation;
mod path_management;
mod persistence;
mod queries;
mod retention;
mod selection;

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::scene::{RuntimeSessionMetadata, RuntimeSessionSlot, World};

fn tagged_slot(
    source: &World,
    slot_id: &str,
    tag: &str,
    updated_at_unix_millis: u64,
) -> RuntimeSessionSlot {
    RuntimeSessionSlot::from_world_with_metadata(
        slot_id,
        source,
        RuntimeSessionMetadata::default()
            .with_tag(tag)
            .with_updated_at_unix_millis(updated_at_unix_millis),
    )
    .expect("tagged slot should capture")
}

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_scene_{label}_{unique}"))
}

fn temporary_archive_leftovers(parent: &Path) -> Vec<String> {
    fs::read_dir(parent)
        .expect("session directory should be readable")
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|file_name| file_name.ends_with(".tmp") || file_name.ends_with(".bak"))
        .collect()
}
