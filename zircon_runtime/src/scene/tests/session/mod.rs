//! 会话归档测试用共享槽位与临时路径夹具比较内存和路径入口的契约，
//! 包括预览、提交、选择、留存和读写产物。

#[path = "archive_artifact.rs"]
mod archive_artifact;
#[path = "capture.rs"]
mod capture;
#[path = "load.rs"]
mod load;
#[path = "merge.rs"]
mod merge;
#[path = "mutation.rs"]
mod mutation;
#[path = "path_management.rs"]
mod path_management;
#[path = "persistence.rs"]
mod persistence;
#[path = "queries.rs"]
mod queries;
#[path = "retention.rs"]
mod retention;
#[path = "selection.rs"]
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
