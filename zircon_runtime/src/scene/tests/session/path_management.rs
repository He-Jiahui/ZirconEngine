//! 磁盘归档的单槽导入、复制与变更从路径入口分组验证；
//! 预览供调用方决定操作，原子替换负责发布完整结果。

use std::fs;

use crate::scene::{
    RuntimeSessionArchive, RuntimeSessionArchiveMergePolicy, RuntimeSessionMetadata, World,
};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

#[path = "path_management/archive_merge.rs"]
mod archive_merge;
#[path = "path_management/mutation_previews.rs"]
mod mutation_previews;
#[path = "path_management/single_slot_import.rs"]
mod single_slot_import;
#[path = "path_management/single_slot_save.rs"]
mod single_slot_save;
#[path = "path_management/slot_copy.rs"]
mod slot_copy;
#[path = "path_management/slot_mutations.rs"]
mod slot_mutations;
